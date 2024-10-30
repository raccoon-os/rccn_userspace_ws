use std::{thread::{self, JoinHandle}, net::SocketAddr};
use crossbeam_channel::{bounded, Receiver, Sender};
use thiserror::Error;
use crate::{
    config::VirtualChannel,
    types::{VirtualChannelInMap, VirtualChannelOutMap}
};
use super::{
    ros2::{Ros2ReaderConfig, Ros2TransportError, Ros2TransportHandler}, udp::UdpTransportHandler, RxTransport, TransportHandler, TransportResult, TxTransport
};

#[derive(Error, Debug)]
pub enum TransportManagerError {
    #[error("ROS2 transport error: {0}")]
    Ros2Error(#[from] Ros2TransportError),
    #[error("Address parse error: {0}")]
    AddrParse(std::net::AddrParseError),
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

pub struct TransportManager {
    udp_handler: UdpTransportHandler,
    ros2_handler: Ros2TransportHandler,
    vc_in_map: VirtualChannelInMap,
    vc_out_map: VirtualChannelOutMap,
}

impl TransportManager {
    pub fn new(ros2_node_prefix: String) -> Result<Self, TransportManagerError> {
        Ok(Self {
            udp_handler: UdpTransportHandler::new(),
            ros2_handler: Ros2TransportHandler::new(ros2_node_prefix)?,
            vc_in_map: VirtualChannelInMap::new(),
            vc_out_map: VirtualChannelOutMap::new(),
        })
    }

    pub fn add_virtual_channel(&mut self, vc: &VirtualChannel) -> Result<(), TransportManagerError> {
        // Setup input direction
        if let Some(tx_transport) = &vc.tx_transport {
            let (vc_in_tx, vc_in_rx) = bounded(32);
            
            match tx_transport {
                TxTransport::Udp(addr) => {
                    let addr: SocketAddr = addr.listen.parse()
                        .map_err(|e| TransportManagerError::AddrParse(e))?;
                    self.add_udp_writer(vc_in_rx, addr);
                }
                TxTransport::Ros2(ros2_transport) => {
                    self.add_ros2_writer(vc_in_rx, ros2_transport.topic_pub.clone());
                }
            }
            
            self.vc_in_map.insert(vc.id, vc_in_tx);
        }

        // Setup output direction
        if let Some(rx_transport) = &vc.rx_transport {
            let (vc_out_tx, vc_out_rx) = bounded(32);
            
            match rx_transport {
                RxTransport::Udp(addr) => {
                    let addr: SocketAddr = addr.send.parse()
                        .map_err(|e| TransportManagerError::AddrParse(e))?;
                    self.add_udp_reader(vc_out_tx, addr);
                }
                RxTransport::Ros2(ros2_transport) => {
                    let reader_config = if let Some(topic) = &ros2_transport.topic_sub {
                        Ros2ReaderConfig::Subscription(topic.clone())
                    } else if let Some(action_srv) = &ros2_transport.action_srv {
                        Ros2ReaderConfig::ActionServer(action_srv.clone())
                    } else {
                        return Err(TransportManagerError::InvalidConfig(
                            "ROS2 rx transport needs either topic_sub or action_srv".into()
                        ));
                    };

                    self.add_ros2_reader(vc_out_tx, reader_config);
                }
            }
            
            self.vc_out_map.insert(vc.id, vc_out_rx);
        }

        Ok(())
    }

    pub fn get_vc_maps(&self) -> (VirtualChannelInMap, VirtualChannelOutMap) {
        (self.vc_in_map.clone(), self.vc_out_map.clone())
    }

    pub fn add_udp_reader(&mut self, tx: Sender<Vec<u8>>, addr: std::net::SocketAddr) {
        self.udp_handler.add_transport_reader(tx, addr);
    }

    pub fn add_udp_writer(&mut self, rx: Receiver<Vec<u8>>, addr: std::net::SocketAddr) {
        self.udp_handler.add_transport_writer(rx, addr);
    }

    pub fn add_ros2_reader(&mut self, tx: Sender<Vec<u8>>, config: Ros2ReaderConfig) {
        self.ros2_handler.add_transport_reader(tx, config);
    }

    pub fn add_ros2_writer(&mut self, rx: Receiver<Vec<u8>>, topic: String) {
        self.ros2_handler.add_transport_writer(rx, topic);
    }

    pub fn run(self) -> Vec<JoinHandle<TransportResult>> {
        vec![
            thread::spawn(move || self.udp_handler.run()),
            thread::spawn(move || self.ros2_handler.run()),
        ]
    }
}
