use std::{
    sync::{Arc, Mutex},
    thread::{self},
    time::Duration,
};

use async_std::stream::StreamExt;
use crossbeam_channel::{Receiver, Select, Sender};
use futures::{executor::LocalPool, task::SpawnExt};
use r2r::{rccn_usr_msgs::msg::RawBytes, QosProfile};
use thiserror::Error;

use super::{TransportHandler, TransportReader, TransportResult, TransportWriter};

#[allow(dead_code)] // Inner value is not read
#[derive(Error, Debug)]
pub enum Ros2TransportError {
    #[error("R2R error {0}")]
    R2RError(r2r::Error),
}

#[allow(dead_code)] // ActionServer not yet implemented
#[derive(Debug)]
pub enum Ros2ReaderConfig {
    Subscription(String),
    ActionServer(String),
}

pub struct Ros2TransportHandler {
    node: Arc<Mutex<r2r::Node>>,
    publishers: Vec<TransportWriter<String>>,
    readers: Vec<TransportReader<Ros2ReaderConfig>>,
    name_prefix: String,
}

impl Ros2TransportHandler {
    pub fn new(name_prefix: String) -> Result<Self, Ros2TransportError> {
        let ctx = r2r::Context::create().map_err(Ros2TransportError::R2RError)?;
        let node = Arc::new(Mutex::new(
            r2r::Node::create(ctx, &name_prefix, "/").map_err(Ros2TransportError::R2RError)?,
        ));

        Ok(Self {
            node,
            publishers: Vec::new(),
            readers: Vec::new(),
            name_prefix,
        })
    }
}

impl TransportHandler for Ros2TransportHandler {
    type WriterConfig = String; // Topic to publish on
    type ReaderConfig = Ros2ReaderConfig; // This one is a bit more complicated.
                                          // We can either subscribe to a topic, or start
                                          // an action server.

    fn add_transport_writer(&mut self, rx: Receiver<Vec<u8>>, config: Self::WriterConfig) {
        let topic: String = config;

        self.publishers.push(TransportWriter { rx, conf: topic });
    }

    fn add_transport_reader(&mut self, tx: Sender<Vec<u8>>, conf: Self::ReaderConfig) {
        self.readers.push(TransportReader { tx, conf });
    }

    fn run(self) -> TransportResult {
        let node_clone = self.node.clone();
        let _readers_handle = thread::spawn(move || run_ros2_readers(node_clone, &self.readers));

        let node_clone = self.node.clone();
        let _spinner_handle = thread::spawn(move || loop {
            node_clone
                .lock()
                .unwrap()
                .spin_once(Duration::from_millis(100));
        });

        if self.publishers.len() == 0 {
            return Ok(());
        }

        let mut select = Select::new();
        let mut publishers = Vec::new();

        for TransportWriter { rx, conf } in self.publishers.iter() {
            select.recv(rx);

            let publisher = self
                .node
                .lock()
                .unwrap()
                .create_publisher::<RawBytes>(conf, QosProfile::default())
                .unwrap();

            publishers.push(publisher);
        }

        loop {
            let op = select.select();
            let index = op.index();

            let TransportWriter { rx, conf: _ } = &self.publishers[index];
            let publisher = &publishers[index];
            match op.recv(rx) {
                Ok(data) => {
                    println!("Got data on channel {}, publishing to topic.", index);

                    let mut msg = RawBytes::default();
                    msg.data = data;
                    match publisher.publish(&msg) {
                        Ok(()) => {
                            println!("Published successfully.")
                        }
                        Err(e) => {
                            println!("Error publishing data to topic: {:?}", e);
                        }
                    }
                }
                Err(_) => todo!(),
            }
        }
    }
}

fn run_ros2_readers(node: Arc<Mutex<r2r::Node>>, readers: &Vec<TransportReader<Ros2ReaderConfig>>) {
    if readers.len() == 0 {
        return;
    }

    let mut pool = LocalPool::new();
    let spawner = pool.spawner();

    for TransportReader { conf, tx } in readers.iter() {
        match conf {
            Ros2ReaderConfig::Subscription(topic) => {
                let tx = tx.clone();
                let topic = topic.clone();

                let mut subscription = node
                    .lock()
                    .unwrap()
                    .subscribe::<RawBytes>(&topic, QosProfile::default())
                    .unwrap();

                let _ = spawner
                    .spawn(async move {
                        println!("Subscribed to {topic}.");
                        loop {
                            match subscription.next().await {
                                Some(msg) => {
                                    println!("Received message on topic {topic}.");
                                    if let Err(e) = tx.send(msg.data) {
                                        println!(
                                            "Error sending message to transmitter, exiting. {e:?}"
                                        );
                                        break;
                                    }
                                }
                                None => todo!(),
                            }
                        }
                    })
                    .unwrap();
            }
            Ros2ReaderConfig::ActionServer(_) => todo!(),
        }
    }

    pool.run();
}
