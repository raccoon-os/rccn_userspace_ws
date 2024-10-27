use crossbeam_channel::{Receiver, Select, Sender};
use futures::{executor::LocalPool, task::SpawnExt};
use r2r::{rccn_usr_msgs::msg::RawBytes, Publisher, QosProfile};

use super::{TransportHandler, TransportWriter};

#[derive(Debug)]
pub enum Ros2TransportError {
    R2RError(r2r::Error),
}
pub struct Ros2TransportHandler {
    ctx: r2r::Context,
    node: r2r::Node,
    publishers: Vec<TransportWriter<Publisher<RawBytes>>>
}

impl Ros2TransportHandler {
    pub fn new() -> Result<Self, Ros2TransportError> {
        let ctx = r2r::Context::create().map_err(Ros2TransportError::R2RError)?;
        let node = r2r::Node::create(ctx.clone(), "rccn_usr_comm", "/")
            .map_err(Ros2TransportError::R2RError)?;

        Ok(Self { ctx, node, publishers: Vec::new() })
    }
}

impl TransportHandler for Ros2TransportHandler {
    type Config = String;

    fn add_transport_writer(&mut self, rx: Receiver<Vec<u8>>, config: Self::Config) {
        let topic: String = config;
        let subscription = self.node.create_publisher::<RawBytes>(&topic, QosProfile::default()).unwrap();
        self.publishers.push(TransportWriter { rx, conf: subscription });
    }

    fn add_transport_reader(&mut self, tx: Sender<Vec<u8>>, config: Self::Config) {
        todo!()
    }

    fn run(self) -> super::TransportResult {
        let mut select = Select::new();
        for TransportWriter { rx, conf: _ } in self.publishers.iter() {
            select.recv(rx);
        };

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
