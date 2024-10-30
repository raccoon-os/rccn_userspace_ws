use std::thread;

use async_std::stream::StreamExt;
use crossbeam_channel::{Receiver, Select, Sender};
use futures::{executor::LocalPool, task::SpawnExt};
use r2r::{rccn_usr_msgs::msg::RawBytes, Publisher, QosProfile};
use thiserror::Error;

use super::{TransportHandler, TransportReader, TransportWriter};

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
    ctx: r2r::Context,
    node: r2r::Node,
    publishers: Vec<TransportWriter<Publisher<RawBytes>>>,
    readers: Vec<TransportReader<Ros2ReaderConfig>>,
    name_prefix: String,
}

impl Ros2TransportHandler {
    pub fn new(name_prefix: String) -> Result<Self, Ros2TransportError> {
        let ctx = r2r::Context::create().map_err(Ros2TransportError::R2RError)?;
        let node = r2r::Node::create(
            ctx.clone(),
            format!("{}_publishers", name_prefix).as_str(),
            "/",
        )
        .map_err(Ros2TransportError::R2RError)?;

        Ok(Self {
            ctx: ctx.clone(),
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
        let publisher = self
            .node
            .create_publisher::<RawBytes>(&topic, QosProfile::default())
            .unwrap();

        self.publishers.push(TransportWriter {
            rx,
            conf: publisher,
        });
    }

    fn add_transport_reader(&mut self, tx: Sender<Vec<u8>>, conf: Self::ReaderConfig) {
        self.readers.push(TransportReader { tx, conf });
    }

    fn run(self) -> super::TransportResult {
        let _readers_handle = thread::spawn(move || {
            run_ros2_readers(self.ctx.clone(), self.name_prefix, &self.readers)
        });

        let mut select = Select::new();
        for TransportWriter { rx, conf: _ } in self.publishers.iter() {
            select.recv(rx);
        }

        loop {
            let op = select.select();
            let index = op.index();

            let TransportWriter { rx, conf: ros2_pub } = &self.publishers[index];
            match op.recv(rx) {
                Ok(data) => {
                    println!("Got data on channel {}, publishing to topic.", index);

                    let mut msg = RawBytes::default();
                    msg.data = data;
                    match ros2_pub.publish(&msg) {
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

fn run_ros2_readers(
    ctx: r2r::Context,
    name_prefix: String,
    readers: &Vec<TransportReader<Ros2ReaderConfig>>,
) {
    let mut pool = LocalPool::new();
    let spawner = pool.spawner();

    let mut node =
        r2r::Node::create(ctx, format!("{}_receivers", name_prefix).as_str(), "/").unwrap();

    for TransportReader { conf, tx } in readers.iter() {
        match conf {
            Ros2ReaderConfig::Subscription(topic) => {
                let tx = tx.clone();
                let topic = topic.clone();

                let mut subscription = node
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

    loop {
        node.spin_once(std::time::Duration::from_millis(100));
        pool.run_until_stalled();
    }
}
