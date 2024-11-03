use rccn_usr::{
    service::{
        AcceptanceResult, AcceptedTc, PusService, PusServiceBase,
    },
    types::VirtualChannelTxMap,
};
use crate::stress_service::command::StressServiceCommand;

pub struct StressTestService {
    service_base: PusServiceBase,
}

impl StressTestService {
    pub fn new(apid: u16, vc_map: &VirtualChannelTxMap) -> Self {
        Self {
            service_base: PusServiceBase::new(apid, 142, 12345u64, vc_map),
        }
    }
}

impl PusService for StressTestService {
    type CommandT = StressServiceCommand;

    fn get_service_base(&mut self) -> PusServiceBase {
        self.service_base.clone()
    }

    fn handle_tc(&mut self, tc: AcceptedTc, cmd: Self::CommandT) -> AcceptanceResult {
        let base = self.get_service_base();

        tc.handle(|| {
            println!("Stress service command {:?}", cmd);
            true
        })
    }
}
