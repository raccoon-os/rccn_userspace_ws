use std::sync::{Arc, Mutex};

use rccn_usr::{service::PusService, transport::{ros2::SharedNode, TransportManager}, types::{VirtualChannelRxMap, VirtualChannelTxMap}};

use crate::parameter_management_service::{PusParameters, SharedPusParameters};


pub struct PusApp<'a> {
    transport_manager: TransportManager,
    node: SharedNode,
    parameters: SharedPusParameters,

    services: Vec<&'a dyn PusService>
}

impl PusApp {
    pub fn new(node: SharedNode, parameters: SharedPusParameters) -> Self {
        Self {
            transport_manager: TransportManager::new("foo".into()).unwrap(),
            parameters: parameters,
            node: node
        }
    }
}