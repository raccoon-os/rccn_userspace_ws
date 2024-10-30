use serde::{Deserialize, Serialize};
use crate::{types::VcId, transport::{TxTransport, RxTransport}};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VirtualChannel {
    pub id: VcId,
    pub name: String,
    pub splitter: Option<String>,
    pub tx_transport: Option<TxTransport>,
    pub rx_transport: Option<RxTransport>
}
