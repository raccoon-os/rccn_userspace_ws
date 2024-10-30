use serde::{Deserialize, Serialize};
use crate::types::VcId;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct UdpTxTransport {
    pub listen: String
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct UdpRxTransport {
    pub send: String
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Ros2TxTransport {
    pub topic_pub: String
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Ros2RxTransport {
    pub topic_sub: Option<String>,
    pub action_srv: Option<String>
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum TxTransport {
    Udp(UdpTxTransport),
    Ros2(Ros2TxTransport)
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum RxTransport {
    Udp(UdpRxTransport),
    Ros2(Ros2RxTransport)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VirtualChannel {
    pub id: VcId,
    pub name: String,
    pub splitter: Option<String>,
    pub tx_transport: Option<TxTransport>,
    pub rx_transport: Option<RxTransport>
}
