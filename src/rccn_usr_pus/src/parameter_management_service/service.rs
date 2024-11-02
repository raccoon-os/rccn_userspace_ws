use std::sync::{Arc, Mutex};

use rccn_usr::service::{AcceptanceResult, CommandExecutionStatus, PusService, PusServiceBase};
use satrs::{
    pus::verification::{TcStateAccepted, VerificationToken},
    spacepackets::ecss::EcssEnumU8,
};
use xtce_rs::bitbuffer::BitWriter;

use super::{command::Command, ParameterError, PusParameters};

type SharedPusParameters = Arc<Mutex<dyn PusParameters + Send>>;

pub struct ParameterManagementService {
    base: PusServiceBase,
    parameters: SharedPusParameters,
}

impl ParameterManagementService {
    pub fn new(base: PusServiceBase, parameters: SharedPusParameters) -> Self {
        Self { base, parameters }
    }

    fn report_parameter_values(
        &mut self,
        n: u16,
        hashes: &Vec<u32>,
    ) -> Result<Vec<u8>, ParameterError> {
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

        Ok(Vec::from(&data[0..bytes_written]))
    }
    fn set_parameter_values(&self, number_of_parameters: &u16, parameter_data: &Vec<u8>) -> bool {
        false
    }
}

impl PusService for ParameterManagementService {
    type CommandT = Command;

    fn get_service_base(&mut self) -> PusServiceBase {
        self.base.clone()
    }

    fn handle_tc(
        &mut self,
        tc: &Self::CommandT,
        token: VerificationToken<TcStateAccepted>,
    ) -> AcceptanceResult {
        let base = self.get_service_base();
        match tc {
            Command::ReportParameterValues {
                number_of_parameters,
                parameter_hashes,
            } => {
                // Make sure the command is properly constructed.
                if *number_of_parameters != parameter_hashes.len() as u16 {
                    base.send_start_failure(token, &EcssEnumU8::new(0), &[])
                        .unwrap();

                    return Ok(CommandExecutionStatus::Failed);
                }

                handle_tc_with_tm!(
                    self.report_parameter_values(*number_of_parameters, parameter_hashes),
                    1
                )
            }
            Command::SetParameterValues {
                number_of_parameters,
                parameter_set_data,
            } => handle_simple_tc!(
                self.set_parameter_values(number_of_parameters, parameter_set_data)
            )
        }
    }
}

/*
fn handle_tc(
    &mut self,
    tc: &Self::CommandT,
    token: VerificationToken<TcStateAccepted>,
) -> ServiceResult<()> {
    let base = self.get_service_base().clone();

    match tc {
        Command::ReportParameterValues { number_of_parameters, parameter_hashes } => {
            let token = base.send_start_success(token).unwrap();
            let result = self.report_parameter_values(number_of_parameters, parameter_hashes);
            if result {
                base.send_completion_success(token).unwrap();
            } else {
                base.send_completion_failure(token, &EcssEnumU8::new(0), &[]).unwrap();
            }
        },
        Command::SetParameterValues { number_of_parameters, parameter_set_data } => {
            let token = base.send_start_success(token).unwrap();
            let result = self.set_parameter_values(number_of_parameters, parameter_set_data);
            if result {
                base.send_completion_success(token).unwrap();
            } else {
                base.send_completion_failure(token, &EcssEnumU8::new(0), &[]).unwrap();
            }
        }
    }

    Ok(())
}*/
