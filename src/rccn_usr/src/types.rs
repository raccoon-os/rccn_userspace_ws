use std::{collections::HashMap, sync::{Arc, Mutex}};

use satrs::{pus::{EcssTmSender, EcssTmtcError, PusTmVariant}, spacepackets::ecss::WritablePusPacket, ComponentId};



pub type Sender = crossbeam_channel::Sender<Vec<u8>>;
pub type Receiver = crossbeam_channel::Receiver<Vec<u8>>;

pub type VcId = u8;
pub type VirtualChannelTxMap = HashMap<VcId, Sender>;
pub type VirtualChannelRxMap = HashMap<VcId, Receiver>;

pub struct RccnEcssTmSender {
    pub channel: Sender,
    pub msg_counter: Arc<Mutex<u16>>
}

impl EcssTmSender for RccnEcssTmSender {
    fn send_tm(&self, _sender_id: ComponentId, tm: PusTmVariant) -> Result<(), EcssTmtcError> {
        let mut tm_creator = match tm {
            PusTmVariant::InStore(_) => todo!(),
            PusTmVariant::Direct(creator) => creator
        }; 

        // CCSDS seq count is updated by the comm application

        // Update PUS service message counter
        { 
            // New block to limit the lifetime of the mutex guard
            let mut counter = self.msg_counter.lock().unwrap();
            tm_creator.set_msg_counter(*counter);
            *counter += 1;
        }

        let bytes = tm_creator.to_vec()?;

        match self.channel.send(bytes) {
            Ok(()) => Ok(()),
            Err(_) => {
                Err(EcssTmtcError::CantSendDirectTm)
            }
        }
    }
}