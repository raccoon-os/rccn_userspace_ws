use std::sync::{Arc, Mutex};

use rccn_usr::service::{PusService, PusServiceBase, ServiceResult};
use satrs::{pus::verification::{TcStateAccepted, VerificationToken}, spacepackets::ecss::EcssEnumU8};

use super::{command::Command, PusParameters};

type SharedPusParameters = Arc<Mutex<dyn PusParameters + Send>>;

pub struct ParameterManagementService {
    base: PusServiceBase,
    parameters: SharedPusParameters
}

impl ParameterManagementService {
    pub fn new(base: PusServiceBase, parameters: SharedPusParameters) -> Self {
        Self {
            base,
            parameters
        }
    }

    fn report_parameter_values(&self, number_of_parameters: &u16, parameter_hashes: &Vec<u32>) -> bool {

        true

    }
    fn set_parameter_values(&self, number_of_parameters: &u16, parameter_data: &Vec<u8>) -> bool {

        false
    }
}

impl PusService for ParameterManagementService {
    type CommandT = Command;

    fn get_service_base(&mut self) -> &mut PusServiceBase {
        &mut self.base
    }

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
    }
}

