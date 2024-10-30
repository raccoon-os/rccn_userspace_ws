use std::collections::HashMap;

use crossbeam_channel::Sender;
use crossbeam_channel::Receiver;

pub type VcId = u8;
pub type VirtualChannelInMap = HashMap<VcId, Sender<Vec<u8>>>;
pub type VirtualChannelOutMap = HashMap<VcId, Receiver<Vec<u8>>>;