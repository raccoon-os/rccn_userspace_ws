use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct UdpTxTransport {
    pub send: String
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct UdpRxTransport {
    pub listen: String
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Ros2TxTransport {
    pub topic_pub: String
}

impl From<&str> for Ros2TxTransport {
    fn from(topic: &str) -> Self {
        Self {
            topic_pub: topic.to_string()
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Ros2RxTransport {
    pub topic_sub: Option<String>,
    pub action_srv: Option<String>
}

impl Ros2RxTransport {
    pub fn with_topic(topic: &str) -> Self {
        Self {
            topic_sub: Some(topic.to_string()),
            action_srv: None
        }
    }

    pub fn with_action(action: &str) -> Self {
        Self {
            topic_sub: None,
            action_srv: Some(action.to_string())
        }
    }
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
