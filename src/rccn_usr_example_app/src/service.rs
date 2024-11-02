use rccn_usr::{
    service::{
        AcceptanceResult, CommandExecutionStatus, PusService, PusServiceBase
    },
    types::VirtualChannelTxMap,
};
use satrs::{
    pus::verification::TcStateAccepted,
    pus::verification::VerificationToken,
};

use crate::command::ExampleServiceCommand;

pub struct ExampleService {
    service_base: PusServiceBase,
}

impl ExampleService {
    pub fn new(apid: u16, vc_map: &VirtualChannelTxMap) -> Self {
        Self {
            service_base: PusServiceBase::new(apid, 142, 12345u64, vc_map),
        }
    }
}

impl PusService for ExampleService {
    type CommandT = ExampleServiceCommand;

    fn get_service_base(&mut self) -> PusServiceBase {
        self.service_base.clone()
    }

    fn handle_tc(
        &mut self,
        tc: &Self::CommandT,
        token: VerificationToken<TcStateAccepted>,
    ) -> AcceptanceResult {
        let base = self.get_service_base();

        match tc {
            ExampleServiceCommand::StartSdrRecording {
                center_freq_hz,
                bandwidth,
                duration_seconds,
            } => {
                let token = base.send_start_success(token).unwrap();
                println!("SDR magic goes here {center_freq_hz} {bandwidth} {duration_seconds}");
                
                base.send_completion_success(token).unwrap();
            }
            ExampleServiceCommand::GenerateRandomFile { filename } => {
                println!("Write random data to {filename}");
            }
        }

        Ok(CommandExecutionStatus::Completed)
    }
}
