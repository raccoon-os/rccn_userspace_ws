//! Implementation of ECSS PUS Service 20 - Parameter Management Service
//! 
//! This service provides the capability to:
//! - Report parameter values (subservice 1)
//! - Set parameter values (subservice 2)
//! 
//! The service operates on parameters that implement the `PusParameters` trait,
//! which provides methods to serialize/deserialize parameter values to/from bytes.
//! Each parameter is identified by a unique 32-bit hash.

use std::sync::{Arc, Mutex};

use rccn_usr::service::{
    AcceptanceResult, AcceptedTc, CommandExecutionStatus, PusService, PusServiceBase,
    SubserviceTmData,
};
use satrs::spacepackets::ecss::EcssEnumU8;
use xtce_rs::bitbuffer::{BitBuffer, BitWriter};

use super::{command::Command, ParameterError, PusParameters};

type SharedPusParameters = Arc<Mutex<dyn PusParameters + Send>>;

/// Implementation of ECSS PUS Service 20 - Parameter Management Service
/// 
/// This service allows reading and writing spacecraft parameters through two subservices:
/// 
/// - Subservice 1 (Report Parameter Values):
///   Generates a report containing the current values of requested parameters
///   
/// - Subservice 2 (Set Parameter Values): 
///   Updates parameter values based on provided data
///
/// # Example
/// ```
/// use rccn_usr::service::PusServiceBase;
/// use std::sync::{Arc, Mutex};
/// 
/// #[derive(PusParameters)]
/// struct MyParams {
///     #[hash(0x1234)]
///     temperature: f32,
/// }
/// 
/// let params = Arc::new(Mutex::new(MyParams { temperature: 20.5 }));
/// let service = ParameterManagementService::new(
///     PusServiceBase::new(1, 20, 0, &vc_map),
///     params
/// );
/// ```
pub struct ParameterManagementService {
    base: PusServiceBase,
    parameters: SharedPusParameters,
}

impl ParameterManagementService {
    /// Creates a new Parameter Management Service instance
    ///
    /// # Arguments
    /// * `base` - Base PUS service configuration 
    /// * `parameters` - Thread-safe reference to parameters that implement PusParameters
    pub fn new(base: PusServiceBase, parameters: SharedPusParameters) -> Self {
        Self { base, parameters }
    }

    /// Generates a report containing the current values of requested parameters
    ///
    /// # Arguments
    /// * `n` - Number of parameters to report
    /// * `hashes` - List of parameter hashes to include in report
    ///
    /// # Returns
    /// * `Ok(SubserviceTmData)` - TM packet containing parameter values
    /// * `Err(ParameterError)` - If parameter not found or serialization fails
    fn report_parameter_values(
        &mut self,
        n: u16,
        hashes: &Vec<u32>,
    ) -> Result<SubserviceTmData, ParameterError> {
        let mut data = [0u8; 512];
        let mut writer = BitWriter::wrap(&mut data);
        let mut bits_written: usize = 0;
        let params = self.parameters.lock().unwrap();

        writer
            .write_bits(n.into(), 16)
            .map_err(ParameterError::WriteError)
            .map(|_| bits_written += 16)?;

        for i in 0..n {
            let hash = hashes[i as usize];
            writer
                .write_bits(hash.into(), 32)
                .map_err(ParameterError::WriteError)
                .map(|_| bits_written += 32)?;

            params
                .get_parameter_as_be_bytes(hash, &mut writer)
                .map(|bits| bits_written += bits)?;
        }
        let bytes_written = bits_written.div_ceil(8);

        Ok(SubserviceTmData {
            subservice: 1,
            data: Vec::from(&data[0..bytes_written]),
        })
    }
    /// Updates parameter values from provided data
    ///
    /// # Arguments
    /// * `n` - Number of parameters to update
    /// * `parameter_set_data` - Raw bytes containing new parameter values
    ///
    /// # Returns
    /// * `true` if all parameters were updated successfully
    /// * `false` if any parameter update failed
    fn set_parameter_values(&self, n: u16, parameter_set_data: &Vec<u8>) -> bool {
        let mut bb = BitBuffer::wrap(&parameter_set_data);
        let mut params = self.parameters.lock().unwrap();

        for _ in 0..n {
            let hash = bb.get_bits(32) as u32;

            if !params.set_parameter_from_be_bytes(hash, &mut bb) {
                return false;
            }
        }
        true
    }
}

impl PusService for ParameterManagementService {
    type CommandT = Command;

    fn get_service_base(&mut self) -> PusServiceBase {
        self.base.clone()
    }

    fn handle_tc(&mut self, mut tc: AcceptedTc, cmd: Self::CommandT) -> AcceptanceResult {
        let base = self.get_service_base();

        match cmd {
            Command::ReportParameterValues {
                number_of_parameters,
                parameter_hashes,
            } => {
                // Make sure the command is properly constructed.
                if number_of_parameters != parameter_hashes.len() as u16 {
                    base.send_start_failure(tc.token, &EcssEnumU8::new(0), &[])
                        .unwrap();

                    return Ok(CommandExecutionStatus::Failed);
                }

                tc.handle_with_tm(|| {
                    self.report_parameter_values(number_of_parameters, &parameter_hashes)
                })
            }
            Command::SetParameterValues {
                number_of_parameters,
                parameter_set_data,
            } => tc.handle(|| self.set_parameter_values(number_of_parameters, &parameter_set_data)),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use crossbeam::channel::bounded;
    use rccn_usr::{
        service::{util::create_pus_tc, CommandExecutionStatus, PusService, PusServiceBase},
        types::{Receiver, VirtualChannelTxMap},
    };
    use rccn_usr_pus_macros::PusParameters;
    use satrs::spacepackets::ecss::{tm::PusTmReader, PusPacket, WritablePusPacket};
    use xtce_rs::bitbuffer::{BitBuffer, BitWriter};

    use crate::parameter_management_service::{src_buffer_to_u64, ParameterError, PusParameters};

    use super::ParameterManagementService;

    #[derive(PusParameters)]
    struct TestParameters {
        #[hash(0xABCDEF00)]
        a: u16,
        #[hash(0x00EFCDAB)]
        b: f32,
        #[hash(0xF00BA400)]
        c: i32,
    }

    #[test]
    fn test_read_end_to_end() {
        let mut common = TestCommon::new(TestParameters {
            a: 0xc0ff,
            b: 1.337,
            c: -42
        });

        let mut tc_data = [0u8; 128];
        let mut tc_buffer = BitWriter::wrap(&mut tc_data);
        tc_buffer.write_bits(3, 16).unwrap();
        tc_buffer.write_bits(0xABCDEF00, 32).unwrap();
        tc_buffer.write_bits(0x00EFCDAB, 32).unwrap();
        tc_buffer.write_bits(0xF00BA400, 32).unwrap();

        let tc = create_pus_tc(1, 20, 1, &tc_data);

        assert_eq!(
            common
                .service
                .handle_tc_bytes(&tc.to_vec().unwrap())
                .unwrap(),
            CommandExecutionStatus::Completed
        );

        // Check verification TM
        common.assert_verif_tm();

        // Check TM header
        let service_tm_bytes = common.tm_rx.try_recv().unwrap();
        let (service_tm, _) = PusTmReader::new(&service_tm_bytes, 8).unwrap();
        assert_eq!(service_tm.service(), 20);
        assert_eq!(service_tm.subservice(), 1);

        // Check TM app data
        let mut reader = BitBuffer::wrap(&service_tm.source_data());
        assert_eq!(reader.get_bits(16), 3); // number of parameters

        // first parameter
        assert_eq!(reader.get_bits(32), 0xABCDEF00);
        assert_eq!(reader.get_bits(16), 0xC0FF);

        // second parameter
        assert_eq!(reader.get_bits(32), 0x00EFCDAB);
        assert_eq!(
            reader.get_bits(32),
            src_buffer_to_u64(&(1.337_f32).to_be_bytes(), 32)
        );

        // third parameter
        assert_eq!(reader.get_bits(32), 0xF00BA400);
        assert_eq!(
            reader.get_bits(32) as i32,
            -42i32
        );
    }

    #[test]
    fn test_write_end_to_end() {
        let mut common = TestCommon::new(TestParameters {
            a: 0xc0ff,
            b: 1.337,
            c: -42
        });

        let mut tc_data = [0u8; 128];
        let mut tc_buffer = BitWriter::wrap(&mut tc_data);
        tc_buffer.write_bits(3, 16).unwrap();
        tc_buffer.write_bits(0xABCDEF00, 32).unwrap();
        tc_buffer.write_bits(0x0000_0000_0000_babe, 64).unwrap();
        tc_buffer.write_bits(0x00EFCDAB, 32).unwrap();
        tc_buffer.write_bytes(&(337.1_f64.to_be_bytes())).unwrap();
        tc_buffer.write_bits(0xF00BA400, 32).unwrap();
        tc_buffer.write_bytes(&(-99i64).to_be_bytes()).unwrap();

        let tc = create_pus_tc(1, 20, 2, &tc_data);

        assert_eq!(
            common
                .service
                .handle_tc_bytes(&tc.to_vec().unwrap())
                .unwrap(),
            CommandExecutionStatus::Completed
        );

        // Check verification TM
        common.assert_verif_tm();

        // Check that the values have been changed
        {
            let parameters = common.parameters.lock().unwrap();
            assert_eq!(parameters.a, 0xBABE);
            assert_eq!(parameters.b, 337.1_f32);
            assert_eq!(parameters.c, -99);
        }
    }

    pub struct TestCommon {
        tm_rx: Receiver,
        service: ParameterManagementService,
        parameters: Arc<Mutex<TestParameters>>
    }



    impl TestCommon {
        fn new(parameters: TestParameters) -> Self {
            let (tm_tx, tm_rx) = bounded(4);
            let mut map = VirtualChannelTxMap::new();
            map.insert(0, tm_tx);

            let shared_parameters = Arc::new(Mutex::new(parameters));

            let service = ParameterManagementService::new(
                PusServiceBase::new(1, 20, 0, &map),
                shared_parameters.clone(),
            );

            TestCommon { tm_rx, service, parameters: shared_parameters }
        }

        fn assert_verif_tm(&mut self) {
            // TODO: currently we just drop 3 TM packets (accepted, started, completed)
            for _ in 0..3 {
                self.tm_rx.try_recv().unwrap();
            }
        }
    }

}
