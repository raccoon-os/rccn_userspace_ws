use rccn_usr::{
    service::{
        AcceptanceResult, AcceptedTc, PusService, PusServiceBase,
    },
    types::VirtualChannelTxMap,
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

    fn handle_tc(&mut self, tc: AcceptedTc, cmd: Self::CommandT) -> AcceptanceResult {
        let base = self.get_service_base();

        match cmd {
            ExampleServiceCommand::StartSdrRecording {
                center_freq_hz,
                bandwidth,
                duration_seconds,
            } => tc.handle(|| {
                println!("SDR magic goes here {center_freq_hz} {bandwidth} {duration_seconds}");
                true
            }),
            ExampleServiceCommand::GenerateRandomFile { filename } => tc.handle(|| {
                println!("Write random data to {filename}");
                true
            })
        }
    }
}
