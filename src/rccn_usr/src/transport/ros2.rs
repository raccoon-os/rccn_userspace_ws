use std::{
    sync::{Arc, Mutex},
    thread::{self},
    time::Duration,
};

use async_std::stream::StreamExt;
use crossbeam_channel::{Receiver, Select, Sender};
use futures::{executor::LocalPool, task::SpawnExt, Stream};
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

pub type SharedNode = Arc<Mutex<r2r::Node>>;

pub struct Ros2TransportHandler {
    node: SharedNode,
    publishers: Vec<TransportWriter<String>>,
    readers: Vec<TransportReader<Ros2ReaderConfig>>,
}

pub fn new_shared_ros2_node(node_name: &str, namespace: &str) -> Result<SharedNode, r2r::Error> {
    let ctx = r2r::Context::create()?;
    Ok(Arc::new(Mutex::new(r2r::Node::create(
        ctx, &node_name, namespace,
    )?)))
}

impl Ros2TransportHandler {
    pub fn new(node_name: &str) -> Result<Self, Ros2TransportError> {
        Self::new_with_node(node_name, None)
    }
    pub fn new_with_node(
        node_name: &str,
        node: Option<SharedNode>,
    ) -> Result<Self, Ros2TransportError> {
        let node = match node {
            Some(n) => n,
            None => {
                new_shared_ros2_node(node_name, &"/").map_err(Ros2TransportError::R2RError)?
            }
        };

        Ok(Self {
            node,
            publishers: Vec::new(),
            readers: Vec::new(),
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
                    log::debug!("Got data on channel {}, publishing to topic.", index);

                    let mut msg = RawBytes::default();
                    msg.data = data;
                    match publisher.publish(&msg) {
                        Ok(()) => {
                            log::debug!("Published successfully.")
                        }
                        Err(e) => {
                            log::error!("Error publishing data to topic: {:?}", e);
                        }
                    }
                }
                Err(_) => todo!(),
            }
        }
    }
}

async fn handle_ros2_topic_subscription(
    topic: String,
    mut subscription: impl Stream<Item = RawBytes> + Unpin,
    tx: Sender<Vec<u8>>,
) {
    log::info!("Subscribed to {topic}.");

    loop {
        match subscription.next().await {
            Some(msg) => {
                log::debug!("Received message on topic {topic}.");
                if let Err(e) = tx.send(msg.data) {
                    log::error!("Error sending message to transmitter, exiting. {e:?}");
                    break;
                }
            }
            None => todo!(),
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

                let subscription = node
                    .lock()
                    .unwrap()
                    .subscribe::<RawBytes>(&topic, QosProfile::default())
                    .unwrap();

                // TODO keep track of whether subscriptions quit
                spawner
                    .spawn(handle_ros2_topic_subscription(topic, subscription, tx))
                    .unwrap();
            }
            Ros2ReaderConfig::ActionServer(_) => todo!(),
        }
    }

    pool.run();
}
