use std::collections::HashMap;



pub type Sender = crossbeam_channel::Sender<Vec<u8>>;
pub type Receiver = crossbeam_channel::Receiver<Vec<u8>>;

pub type VcId = u8;
pub type VirtualChannelTxMap = HashMap<VcId, Sender>;
pub type VirtualChannelRxMap = HashMap<VcId, Receiver>;
