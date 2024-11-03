use crossbeam_channel::{Receiver, Select, Sender};
use futures::{executor::LocalPool, task::SpawnExt};
use std::{
    net::{SocketAddr, UdpSocket},
    thread,
};

use super::{
    TransportError, TransportReader, TransportResult, TransportWriter, TRANSPORT_BUFFER_SIZE,
};

use super::TransportHandler;

pub struct UdpTransportHandler {
    writers: Vec<TransportWriter<SocketAddr>>,
    readers: Vec<TransportReader<SocketAddr>>,
}

impl UdpTransportHandler {
    pub fn new() -> Self {
        Self {
            writers: Vec::new(),
            readers: Vec::new(),
        }
    }
}

impl TransportHandler for UdpTransportHandler {
    type WriterConfig = SocketAddr;
    type ReaderConfig = SocketAddr;

    fn add_transport_writer(&mut self, rx: Receiver<Vec<u8>>, config: Self::WriterConfig) {
        self.writers.push(TransportWriter { rx, conf: config });
    }

    fn add_transport_reader(&mut self, tx: Sender<Vec<u8>>, config: Self::ReaderConfig) {
        self.readers.push(TransportReader { tx, conf: config });
    }

    fn run(self) -> TransportResult {
        let _readers_handle = thread::spawn(move || run_udp_transport_readers(self.readers));
        if self.writers.len() == 0 {
            return Ok(())
        }

        let socket = UdpSocket::bind("0.0.0.0:0").map_err(TransportError::IO)?;

        let mut select = Select::new();

        for TransportWriter { rx, conf: _ } in self.writers.iter() {
            select.recv(rx);
        }

        loop {
            let op = select.select();
            let index = op.index();

            log::debug!("RX channel {index} became available.");

            let TransportWriter { rx, conf: addr } = &self.writers[index];
            match op.recv(rx) {
                Ok(data) => {
                    //println!("Got data {data:?} for addr {:?}", addr);

                    match socket.send_to(&data, addr) {
                        Ok(len) => {
                            log::debug!("Sent {len} bytes to {:?}", addr);
                        }
                        Err(e) => {
                            log::error!("Error sending bytes to {:?}: {e:?}", addr);
                        }
                    }
                }
                Err(e) => {
                    log::error!("Got error receiving from RX channel ID {index}: {e:?}");
                    break Ok(()); // TODO propagate error up
                }
            }
        }
    }
}

fn run_udp_transport_readers(readers: Vec<TransportReader<SocketAddr>>) {
    if readers.len() == 0 {
        return;
    }

    let mut pool = LocalPool::new();
    let spawner = pool.spawner();

    for TransportReader { tx, conf } in readers {
        let bind_addr = conf;
        let tx = tx.clone();

        spawner
            .spawn(async move {
                let mut buf = [0u8; TRANSPORT_BUFFER_SIZE];
                let socket = async_std::net::UdpSocket::bind(bind_addr).await.unwrap();
                log::info!("Listening on {bind_addr:?}.");

                loop {
                    match socket.recv_from(&mut buf).await {
                        Ok((size, _addr)) => {
                            let data_vec = Vec::from(&buf[..size]);

                            if let Err(e) = tx.send(data_vec) {
                                log::error!("Error sending data to channel: {:?}", e);
                                break;
                            } 
                        }
                        Err(e) => {
                            log::error!("Error receiving from socket: {:?}", e);
                            break;
                        }
                    }
                }
            })
            .unwrap();
    }

    pool.run();
}
