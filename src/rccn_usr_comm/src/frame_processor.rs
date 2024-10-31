use ccsds_protocols::traits::CCSDSFrames;
use crossbeam_channel::{Receiver, Select, SendError, Sender};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use ccsds_protocols::tc_transfer_frame::TcTransferFrame;

use crate::config::{Config, FrameKind};
use rccn_usr::types::{VcId, VirtualChannelRxMap, VirtualChannelTxMap};

use ccsds_protocols::uslp_transfer_paket::USLPTransferPaket;

#[allow(dead_code)] // IO and SendError values are not read currently
#[derive(Debug)]
pub enum FrameProcessingError {
    IO(std::io::Error),
    SendError(SendError<Vec<u8>>),
    RXChannelClosed,
    UnknownSpacecraft(u16),
    UnknownVirtualChannel(VcId),
}

pub type FrameProcessingResult = Result<(), FrameProcessingError>;

const FRAME_PROCESSING_BUFFER_SIZE: usize = 8096;

#[derive(Clone)]
pub struct FrameProcessor {
    config: Arc<Config>,
    shared_state: Arc<Mutex<HashMap<String, Vec<u8>>>>,
}

impl FrameProcessor {
    pub fn new(config: Arc<Config>) -> Self {
        Self {
            config,
            shared_state: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn process_incoming_frames(
        &self,
        bytes_in_rx: Receiver<Vec<u8>>,
        vc_in_map: &VirtualChannelTxMap,
    ) -> FrameProcessingResult {
        let _state = Arc::clone(&self.shared_state);

        let mut buf = [0u8; FRAME_PROCESSING_BUFFER_SIZE];
        let mut buf_pos: usize = 0;

        loop {
            let data = bytes_in_rx
                .recv()
                .map_err(|_| FrameProcessingError::RXChannelClosed)?;

            // Check if the data fits.
            let rcvd_size = data.len();
            if rcvd_size > buf_pos + buf.len() {
                println!(
                    "Received data too big, size {} does not fit in buffer (current position: {})",
                    data.len(),
                    buf_pos
                );
                println!("Discarding data and clearing buffer.");
                buf.fill(0);
                buf_pos = 0;
                continue;
            }

            // Copy the data to our buffer
            buf[0..buf_pos + rcvd_size].copy_from_slice(&data);
            buf_pos += rcvd_size;

            // TODO: not possible to use `Option<dyn CCSDSFrames, usize>` because CCSDSFrames has PartialEq
            let frame_result = match self.config.frames.r#in.frame_kind {
                FrameKind::Tc => {
                    // TODO this shouldn't be mut
                    TcTransferFrame::from_bytes(&mut buf)
                }
                FrameKind::Uslp => todo!(),
            };

            if let Ok((frame, size)) = frame_result {
                println!("We got a valid frame: {:?}", frame);

                match self.distribute_vc_data(&frame, vc_in_map) {
                    Ok(()) => {
                        println!("Frame data sent to transport sucessfully.");
                    }
                    Err(FrameProcessingError::UnknownSpacecraft(id)) => {
                        println!("Received frame for unknown spacecraft ID {}", id);
                    }
                    Err(FrameProcessingError::UnknownVirtualChannel(id)) => {
                        println!("Received frame for unknown virtual channel ID {}", id);
                    }
                    Err(e) => {
                        println!("Unexpected error sending VC data: {:?}", e);
                    }
                };

                // Clear buffer up to the end of the frame we received
                buf[0..size].fill(0);
                buf_pos = 0;
            } else if let Err(_e) = frame_result {
                // TODO check potential errors in from_bytes,
                // some may indicate we have a broken frame
                //  and should clear the buffer
            }
        }
    }

    fn distribute_vc_data(
        &self,
        frame: &TcTransferFrame<'_>,
        vc_in_map: &VirtualChannelTxMap,
    ) -> FrameProcessingResult {
        if frame.get_spacecraft_id() != self.config.frames.spacecraft_id {
            return Err(FrameProcessingError::UnknownSpacecraft(
                frame.get_spacecraft_id(),
            ));
        }

        match vc_in_map.get(&frame.get_vc_id()) {
            None => Err(FrameProcessingError::UnknownVirtualChannel(
                frame.get_vc_id(),
            )),
            Some(sender) => {
                let data_vec = Vec::from(frame.get_data_field());

                // TODO: process splitting incoming data stream according to
                // the `splitter` config variable for this virtual channel.

                sender
                    .send(data_vec)
                    .map_err(FrameProcessingError::SendError)
            }
        }
    }

    pub fn process_frames_out(&self, bytes_tx: Sender<Vec<u8>>, vc_out_map: &VirtualChannelRxMap) {
        let mut select = Select::new();
        let mut channels = Vec::new();
        for (id, receiver) in vc_out_map {
            select.recv(receiver);
            channels.push((id, receiver));
        }

        loop {
            // Block until a channel has data ready to be received.
            let op = select.select();
            let index = op.index();

            let (vc_id, channel) = channels[index];
            match op.recv(channel) {
                Ok(data) => {
                    println!("Received data on channel for VC ID {vc_id}: {data:?}");

                    // TODO: Put it into a frame and send it to bytes_tx
                    self.frame_and_send_virtual_channel_data(bytes_tx.clone(), *vc_id, &data);
                }
                Err(_) => todo!(),
            }
        }
    }

    pub fn frame_and_send_virtual_channel_data(&self, bytes_tx: Sender<Vec<u8>>, vc_id: VcId, data: &[u8]) {
        let mut frame: USLPTransferPaket<0, 512> = USLPTransferPaket::construct_final_frame(
            12,
            0xab,
            true,
            vc_id.into(),
            0,
            false,
            522,//data.len() as u16,
            false,
            false,
            0,
            false,
            1,
            1,
            [],
            0,
            0,
            0,
            data,
            0,
            0,
        )
        .expect("USLP creation failed");

        let mut buf = [0u8; 65536];
        match frame.to_bytes(&mut buf) {
            Ok(size) => {
                bytes_tx.send(Vec::from(&buf[0..size])).unwrap();
            }
            Err(_) => todo!(),
        }

    }
}
