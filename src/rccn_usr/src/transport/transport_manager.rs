use std::thread::{self, JoinHandle};
use crossbeam_channel::{Receiver, Sender};
use thiserror::Error;
use super::{
    TransportHandler, TransportResult,
    ros2::{Ros2TransportHandler, Ros2ReaderConfig, Ros2TransportError},
    udp::UdpTransportHandler,
};

#[derive(Error, Debug)]
pub enum TransportManagerError {
    #[error("ROS2 transport error: {0}")]
    Ros2Error(#[from] Ros2TransportError),
}

pub struct TransportManager {
    udp_handler: UdpTransportHandler,
    ros2_handler: Ros2TransportHandler,
}

impl TransportManager {
    pub fn new(ros2_node_prefix: String) -> Result<Self, TransportManagerError> {
        Ok(Self {
            udp_handler: UdpTransportHandler::new(),
            ros2_handler: Ros2TransportHandler::new(ros2_node_prefix)?,
        })
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
