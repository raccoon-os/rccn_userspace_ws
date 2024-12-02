use crossbeam_channel::{Receiver, Select, Sender};
use futures::executor::block_on;
use std::{
    io,
    net::{SocketAddr, UdpSocket},
    thread,
};

use super::{
    TransportError, TransportReader, TransportResult, TransportWriter, TRANSPORT_BUFFER_SIZE,
};

use super::TransportHandler;

pub struct AsyncUdpTransportHandler {}

impl AsyncUdpTransportHandler {
    pub fn new() -> Self {
        Self {}
    }
}

impl TransportHandler for AsyncUdpTransportHandler {
    type WriterConfig = SocketAddr;
    type ReaderConfig = SocketAddr;
    type Error = io::Error;

    async fn add_transport_writer(
        &self,
        rx: Receiver<Vec<u8>>,
        config: Self::WriterConfig,
    ) -> Result<(), Self::Error> {
        let socket = tokio::net::UdpSocket::bind("0.0.0.0:0").await?;
        socket.connect(config).await?;

        tokio::task::spawn_blocking(move || {
            while let Ok(bytes) = rx.recv() {
                block_on(socket.send(&bytes));
            }
        });

        Ok(())
    }

    async fn add_transport_reader(
        &self,
        tx: Sender<Vec<u8>>,
        config: Self::ReaderConfig,
    ) -> Result<(), Self::Error> {

        let socket = tokio::net::UdpSocket::bind(config).await?;
        let mut buf = [0u8; TRANSPORT_BUFFER_SIZE];

        tokio::spawn(async move {
            while let Ok(sz) = socket.recv(&mut buf).await {
                tx.send(buf[..sz].to_vec());
            }
        });

        Ok(())
    }
}
