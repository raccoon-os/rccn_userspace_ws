use crate::{
    transport::{
        RxTransport, TxTransport,
    },
    types::VcId,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VirtualChannel {
    pub id: VcId,
    pub name: String,
    pub tx_transport: Option<TxTransport>,
    pub rx_transport: Option<RxTransport>,
}

impl VirtualChannel {
    pub fn on_z_topic(id: VcId, name: &str) -> Result<Self, zenoh::Error> {
        let tx_topic = format!("vc/{name}/tx");
        let rx_topic = format!("vc/{name}/rx");

        Ok(Self {
            id: id,
            name: name.into(),
            tx_transport: Some(TxTransport::Zenoh(tx_topic.as_str().try_into()?)),
            rx_transport: Some(RxTransport::Zenoh(rx_topic.as_str().try_into()?)),
        })
    }
}
