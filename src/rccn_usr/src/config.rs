use serde::{Deserialize, Serialize};
use crate::types::VcId;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct UdpInputTransport {
    pub listen: String
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct UdpOutputTransport {
    pub send: String
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Ros2InputTransport {
    pub topic_pub: String
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Ros2OutputTransport {
    pub topic_sub: Option<String>,
    pub action_srv: Option<String>
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum InputTransport {
    Udp(UdpInputTransport),
    Ros2(Ros2InputTransport)
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum OutputTransport {
    Udp(UdpOutputTransport),
    Ros2(Ros2OutputTransport)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VirtualChannel {
    pub id: VcId,
    pub name: String,
    pub splitter: Option<String>,
    pub in_transport: Option<InputTransport>,
    pub out_transport: Option<OutputTransport>
}
