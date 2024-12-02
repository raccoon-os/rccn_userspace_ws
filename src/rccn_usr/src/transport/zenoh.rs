use zenoh::{key_expr::OwnedKeyExpr, Session, Wait};

use crate::types::{Receiver, Sender};

use super::TransportHandler;

pub struct ZenohTransportHandler {
    session: Session,
}

impl ZenohTransportHandler {
    pub fn new_with_session(session: Session) -> Self {
        Self { session: session }
    }

    pub async fn new() -> Result<Self, zenoh::Error> {
        let session = zenoh::open(zenoh::Config::default()).await?;
        Ok(Self::new_with_session(session))
    }
}

impl TransportHandler for ZenohTransportHandler {
    type WriterConfig = OwnedKeyExpr;
    type ReaderConfig = OwnedKeyExpr;
    type Error = zenoh::Error;

    async fn add_transport_reader(
        &self,
        tx: Sender,
        key: OwnedKeyExpr,
    ) -> Result<(), zenoh::Error> {
        let subscriber = self.session.declare_subscriber(key).await.unwrap();

        tokio::spawn(async move {
            println!("Zenoh subscriber task started.");
            while let Ok(sample) = subscriber.recv_async().await {
                tx.send(sample.payload().to_bytes().to_vec())
                    .expect("Error sending to TX channel");
            }
            println!("Zenoh subscriber quit.");
        });

        Ok(())
    }

    async fn add_transport_writer(
        &self,
        rx: Receiver,
        key: OwnedKeyExpr,
    ) -> Result<(), zenoh::Error> {
        let session = self.session.clone();
        let publisher = session.declare_publisher(key).await?;

        tokio::task::spawn_blocking(move || {
            while let Ok(bytes) = rx.recv() {
                println!("Zenoh publishing.");
                publisher.put(bytes).wait().expect("Error publishing.");
            }

            println!("Zenoh publisher quit.");
        });

        Ok(())
    }
    
}
