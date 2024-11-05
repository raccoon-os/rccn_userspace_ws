
use crossbeam::channel::Select;
use rccn_usr::{
    config::VirtualChannel, service::{AcceptanceResult, CommandReplyBase, PusAppBase, PusService}, transport::{manager::TransportManagerError, ros2::SharedNode, TransportManager}, types::{Receiver, Sender, VcId}
};

type ServiceHandler = Box<dyn FnMut(&[u8], CommandReplyBase) -> AcceptanceResult + Send>;

pub struct PusApp {
    transport_manager: TransportManager,
    handlers: Vec<(u8, ServiceHandler)>,
    base: PusAppBase,
}

impl PusApp {
    pub fn new(apid: u16, ros2_node_prefix: String) -> Self {
        Self {
            transport_manager: TransportManager::new(ros2_node_prefix).unwrap(),
            handlers: Vec::new(),
            base: PusAppBase::new(apid, 0),
        }
    }

    pub fn new_with_ros2_node(apid: u16, node: SharedNode) -> Self {
        Self {
            transport_manager: TransportManager::new_with_ros2_node(node).unwrap(),
            handlers: Vec::new(),
            base: PusAppBase::new(apid, 0)
        }
    }

    pub fn register_service<S: PusService + 'static + Send>(&mut self, mut service: S) {
        let handler: ServiceHandler =
            Box::new(move |bytes, base| service.handle_tc_bytes(bytes, base));

        self.handlers.push((S::service(), handler));
    }

    pub fn add_virtual_channel(&mut self, vc: &VirtualChannel) -> Result<(), TransportManagerError> {
        self.transport_manager.add_virtual_channel(vc)
    }

    fn handle_tc_internal(
        app_base: &PusAppBase,
        handlers: &mut Vec<(u8, ServiceHandler)>,
        data: &[u8],
        tx: Sender,
    ) -> Vec<AcceptanceResult> {
        handlers
            .iter_mut()
            // Call each service handler
            .map(|(service_id, handler)| {
                let reply_base = app_base.new_reply(*service_id, tx.clone());
                handler(data, reply_base)
            })
            // Gather all the AcceptanceResults
            .collect()
    }

    // Mainly for testing purposes
    pub fn handle_tc(&mut self, data: &[u8], tx: Sender) -> Vec<AcceptanceResult> {
        Self::handle_tc_internal(&self.base, &mut self.handlers, data, tx)
    }

    pub fn run(mut self) {
        // Run transports and get maps of VC IDs to TX/RX channels
        let ((vc_tx_map, vc_rx_map), _handles) = self.transport_manager.run();

        // Vec to track of the VC ID, TX channel and RX channel
        // for each RX we add to the select operation
        let mut vc_info: Vec<(VcId, &Receiver, &Sender)> = Vec::new();

        // Select object to wait on multiple RX channels
        let mut select = Select::new();

        for (vc_id, rx) in vc_rx_map.iter() {
            // Add RX channel for this VC to the select object
            select.recv(rx);

            let tx = vc_tx_map.get(vc_id).unwrap(); // TODO make tx side optional
            vc_info.push((*vc_id, rx, tx));
        }

        loop {
            // Wait until a RX channel is available, get VC info
            let op = select.select();
            let (vc, rx, tx) = vc_info[op.index()];

            // Attempt to receive from the channel
            match op.recv(rx) {
                Ok(msg) => {
                    println!("PUS APP received command on vc id {vc}");

                    Self::handle_tc_internal(&self.base, &mut self.handlers, &msg, tx.clone());

                    // TODO: check that exactly one service handled the command succesfully
                }
                Err(_) => todo!(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use super::*;
    use crate::parameter_management_service::{
        service::ParameterManagementService, ParameterError, PusParameters,
    };
    use crossbeam::channel::bounded;
    use rccn_usr::service::{util::create_pus_tc, CommandExecutionStatus};
    use rccn_usr_pus_macros::PusParameters;
    use satrs::spacepackets::ecss::WritablePusPacket;
    use xtce_rs::bitbuffer::{BitBuffer, BitWriter};

    #[derive(PusParameters)]
    struct TestParameters {
        #[hash(0x1234)]
        value: u32,
    }

    #[test]
    fn test_register_and_handle_service() {
        let (tm_tx, tm_rx) = bounded(4);

        // Create PusApp and register ParameterManagementService
        let mut app = PusApp::new(1, "test".into());
        let parameters = Arc::new(Mutex::new(TestParameters { value: 42 }));
        let service = ParameterManagementService::new(parameters);
        app.register_service(service);

        // Create a test TC for parameter reporting
        let mut tc_data = [0u8; 128];
        tc_data[0] = 0; // Number of parameters MSB
        tc_data[1] = 1; // Number of parameters LSB
        tc_data[2] = 0; // Parameter hash
        tc_data[3] = 0;
        tc_data[4] = 0x12;
        tc_data[5] = 0x34;

        let tc = create_pus_tc(1, 20, 1, &tc_data);
        let tc_bytes = tc.to_vec().unwrap();

        // Handle TC
        let results = app.handle_tc(&tc_bytes, tm_tx);

        // Check that a service returned Completed
        assert!(results
            .iter()
            .any(|r| matches!(r, Ok(CommandExecutionStatus::Completed))));

        // Check that 4 messages were sent to TM rx (accepted, started, completed, parameter TM)
        assert_eq!(tm_rx.len(), 4);
    }
}
