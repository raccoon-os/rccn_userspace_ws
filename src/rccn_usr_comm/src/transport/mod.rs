pub mod udp;
pub mod ros2;

pub use udp::*;

use crossbeam_channel::{SendError, Sender, Receiver};
use std::io;

#[allow(dead_code)] // Inner values not read currently
#[derive(Debug)]
pub enum TransportError {
    IO(io::Error),
    SendError(SendError<Vec<u8>>),
}

pub struct TransportWriter<T> {
    rx: Receiver<Vec<u8>>,
    conf: T,
}

pub struct TransportReader<T> {
    tx: Sender<Vec<u8>>,
    conf: T,
}

pub type TransportResult = Result<(), TransportError>;

pub const TRANSPORT_BUFFER_SIZE: usize = 8096;

pub trait TransportHandler {
    type Config;
    
    fn add_transport_writer(&mut self, rx: Receiver<Vec<u8>>, config: Self::Config);
    fn add_transport_reader(&mut self, tx: Sender<Vec<u8>>, config: Self::Config);
    fn run(self) -> TransportResult;
}
