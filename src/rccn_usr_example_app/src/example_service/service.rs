use std::{thread::sleep, time::Duration};

use rccn_usr::{service::{AcceptanceResult, AcceptedTc, PusService, PusServiceBase}, types::VirtualChannelTxMap};

use super::command;

pub struct ExampleService {
    base: PusServiceBase
}

impl ExampleService {
    pub fn new(apid: u16, vc_map: &VirtualChannelTxMap) -> Self {
        Self {
            base: PusServiceBase::new(apid, 130, 0, vc_map)
        }
    }
}

impl PusService for ExampleService {
    type CommandT = command::Command;

    fn get_service_base(&mut self) -> PusServiceBase {
        self.base.clone()
    }

    fn handle_tc(&mut self, tc: AcceptedTc, cmd: Self::CommandT) -> AcceptanceResult {
        match cmd {
            command::Command::GeneratedCommandTest(args) => tc.handle(||{
                sleep(Duration::from_millis(2000));
                println!("Generated command test args: {args:?}");
                true
            })
        }
    }
}