use std::sync::{Arc, Mutex};

use rccn_usr::{
    service::{AcceptanceResult, CommandReplyBase, PusService},
    transport::TransportManager,
};

type ServiceHandler = Box<dyn Fn(&[u8], CommandReplyBase) -> AcceptanceResult + Send>;

pub struct PusApp {
    transport_manager: TransportManager,
    service_handlers: Vec<ServiceHandler>,
}

impl PusApp {
    pub fn new(ros2_node_prefix: String) -> Self {
        Self {
            transport_manager: TransportManager::new(ros2_node_prefix).unwrap(),
            service_handlers: Vec::new(),
        }
    }

    pub fn register_service<S: PusService + 'static>(&mut self, mut service: S) {
        let handler: ServiceHandler = Box::new(move |bytes, base| service.handle_tc_bytes(bytes, base));
        self.service_handlers.push(handler);
    }

    // Function to handle incoming TC - calls each registered service handler
    pub fn handle_tc(&self, bytes: &[u8], base: CommandReplyBase) -> Vec<AcceptanceResult> {
        self.service_handlers
            .iter()
            .map(|handler| handler(bytes, base.clone()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parameter_management_service::{service::ParameterManagementService, PusParameters};
    use crossbeam::channel::bounded;
    use rccn_usr::service::util::create_pus_tc;
    use rccn_usr_pus_macros::PusParameters;
    use satrs::spacepackets::ecss::WritablePusPacket;

    #[derive(PusParameters)]
    struct TestParameters {
        #[hash(0x1234)]
        value: u32,
    }

    #[test]
    fn test_register_and_handle_service() {
        let (tm_tx, _tm_rx) = bounded(4);
        
        // Create PusApp and register ParameterManagementService
        let mut app = PusApp::new("test".into());
        let parameters = Arc::new(Mutex::new(TestParameters { value: 42 }));
        let service = ParameterManagementService::new(parameters);
        app.register_service(service);

        // Create a test TC for parameter reporting
        let mut tc_data = [0u8; 128];
        tc_data[0] = 0;  // Number of parameters MSB
        tc_data[1] = 1;  // Number of parameters LSB
        tc_data[2] = 0;  // Parameter hash
        tc_data[3] = 0;
        tc_data[4] = 0x12;
        tc_data[5] = 0x34;
        
        let tc = create_pus_tc(1, 20, 1, &tc_data);
        let tc_bytes = tc.to_vec().unwrap();

        // Create reply base for the service
        let reply_base = CommandReplyBase::new(1, 20, 0, tm_tx);

        // Handle TC and check results
        let results = app.handle_tc(&tc_bytes, reply_base);
        assert_eq!(results.len(), 1);
    }
}
