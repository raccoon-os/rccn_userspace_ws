use std::collections::HashMap;

use crossbeam_channel::Sender;
use crossbeam_channel::Receiver;

pub type VcId = u8;
pub type VirtualChannelTxMap = HashMap<VcId, Sender<Vec<u8>>>;
pub type VirtualChannelRxMap = HashMap<VcId, Receiver<Vec<u8>>>;