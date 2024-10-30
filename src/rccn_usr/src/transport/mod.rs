pub mod udp;
pub mod ros2;
pub mod manager;
pub mod config;

use thiserror::Error;
pub use udp::*;
pub use manager::TransportManager;
pub use config::{TxTransport, RxTransport};

use crossbeam_channel::{SendError, Sender, Receiver};
use std::io;

#[allow(dead_code)] // Inner values not read currently
#[derive(Error, Debug)]
pub enum TransportError {
    #[error("IO error {0}")]
    IO(#[from] io::Error),

    #[error("Error sending to channel {0}")]
    SendError(#[from] SendError<Vec<u8>>),
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
    type WriterConfig;
    type ReaderConfig;
    
    fn add_transport_writer(&mut self, rx: Receiver<Vec<u8>>, config: Self::WriterConfig);
    fn add_transport_reader(&mut self, tx: Sender<Vec<u8>>, config: Self::ReaderConfig);
    fn run(self) -> TransportResult;
}
