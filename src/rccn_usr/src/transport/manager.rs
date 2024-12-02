use super::{
    udp::AsyncUdpTransportHandler, zenoh::ZenohTransportHandler, RxTransport, TransportHandler,
    TxTransport,
};
use crate::{
    config::VirtualChannel,
    types::{VirtualChannelRxMap, VirtualChannelTxMap},
};
use crossbeam_channel::bounded;
use std::net::SocketAddr;
use thiserror::Error;
use zenoh::Session;

#[derive(Error, Debug)]
pub enum TransportManagerError {
    #[error("Zenoh error: {0}")]
    ZenohError(#[from] zenoh::Error),
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("Address parse error: {0}")]
    AddrParse(std::net::AddrParseError),
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

pub struct TransportManager {
    pub udp_handler: AsyncUdpTransportHandler,
    zenoh_handler: ZenohTransportHandler,
    vc_tx_map: VirtualChannelTxMap,
    vc_rx_map: VirtualChannelRxMap,
}

impl TransportManager {
    // TODO: Refactor the `new` methods.
    // TransportManager should be able to be used even if not(cfg(ros2))
    pub async fn new() -> Result<Self, TransportManagerError> {
        Ok(Self {
            udp_handler: AsyncUdpTransportHandler::new(),
            zenoh_handler: ZenohTransportHandler::new().await?,
            vc_tx_map: VirtualChannelTxMap::new(),
            vc_rx_map: VirtualChannelRxMap::new(),
        })
    }
    pub fn new_with_z_session(session: Session) -> Result<Self, TransportManagerError> {
        Ok(Self {
            udp_handler: AsyncUdpTransportHandler::new(),
            zenoh_handler: ZenohTransportHandler::new_with_session(session),
            vc_tx_map: VirtualChannelTxMap::new(),
            vc_rx_map: VirtualChannelRxMap::new(),
        })
    }

    pub async fn add_virtual_channel(
        &mut self,
        vc: &VirtualChannel,
    ) -> Result<(), TransportManagerError> {
        // Setup output direction
        if let Some(tx_transport) = &vc.tx_transport {
            let (vc_in_tx, vc_in_rx) = bounded(32);

            match tx_transport {
                TxTransport::Udp(addr) => {
                    let addr: SocketAddr = addr
                        .send
                        .parse()
                        .map_err(|e| TransportManagerError::AddrParse(e))?;

                    self.udp_handler
                        .add_transport_writer(vc_in_rx, addr)
                        .await
                        .map_err(TransportManagerError::IOError)?;
                }
                TxTransport::Zenoh(z_transport) => {
                    self.zenoh_handler
                        .add_transport_writer(vc_in_rx, z_transport.key_pub.clone())
                        .await
                        .map_err(TransportManagerError::ZenohError)?;
                }
            }

            self.vc_tx_map.insert(vc.id, vc_in_tx);
        }

        // Setup input direction
        if let Some(rx_transport) = &vc.rx_transport {
            let (vc_out_tx, vc_out_rx) = bounded(32);

            match rx_transport {
                RxTransport::Udp(addr) => {
                    let addr: SocketAddr = addr
                        .listen
                        .parse()
                        .map_err(|e| TransportManagerError::AddrParse(e))?;

                    self.udp_handler
                        .add_transport_reader(vc_out_tx, addr)
                        .await
                        .map_err(TransportManagerError::IOError)?;
                }
                RxTransport::Zenoh(z_transport) => {
                    self.zenoh_handler
                        .add_transport_reader(vc_out_tx, z_transport.key_sub.clone())
                        .await
                        .map_err(TransportManagerError::ZenohError)?;
                }
            }

            self.vc_rx_map.insert(vc.id, vc_out_rx);
        }

        Ok(())
    }

    pub fn get_vc_maps(&self) -> (&VirtualChannelTxMap, &VirtualChannelRxMap) {
        (&self.vc_tx_map, &self.vc_rx_map)
    }
    pub fn tx_map(&self) -> &VirtualChannelTxMap {
        &self.vc_tx_map
    }
    pub fn rx_map(&self) -> &VirtualChannelRxMap {
        &self.vc_rx_map
    }
}
