use serde::{Deserialize, Serialize};
use zenoh::key_expr::OwnedKeyExpr;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct UdpTxTransport {
    pub send: String
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct UdpRxTransport {
    pub listen: String
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ZenohTxTransport {
    pub key_pub: OwnedKeyExpr
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ZenohRxTransport {
    pub key_sub: OwnedKeyExpr
}


#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum TxTransport {
    Udp(UdpTxTransport),
    Zenoh(ZenohTxTransport)
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum RxTransport {
    Udp(UdpRxTransport),
    Zenoh(ZenohRxTransport)
}


impl TryFrom<&str> for ZenohTxTransport {
    type Error = zenoh::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let kexpr = OwnedKeyExpr::new(value)?;

        Ok(ZenohTxTransport {
            key_pub: kexpr
        })
    }
}

impl TryFrom<&str> for ZenohRxTransport {
    type Error = zenoh::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let kexpr = OwnedKeyExpr::new(value)?;

        Ok(ZenohRxTransport {
            key_sub: kexpr
        })
    }
}