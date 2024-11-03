use crate::{
    transport::{config::Ros2RxTransport, RxTransport, TxTransport},
    types::VcId,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VirtualChannel {
    pub id: VcId,
    pub name: String,
    pub splitter: Option<String>,
    pub tx_transport: Option<TxTransport>,
    pub rx_transport: Option<RxTransport>,
}

impl VirtualChannel {
    pub fn new_ros2(name: &str, id: VcId) -> Self {
        let tx_topic = format!("/vc/{name}/tx");
        let rx_topic = format!("/vc/{name}/rx");

        Self {
            id: id,
            name: name.into(),
            splitter: None,
            tx_transport: Some(TxTransport::Ros2(tx_topic.as_str().into())),
            rx_transport: Some(RxTransport::Ros2(Ros2RxTransport::with_topic(&rx_topic))),
        }
    }
}
