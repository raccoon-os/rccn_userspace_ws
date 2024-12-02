use crossbeam_channel::bounded;
use std::{sync::Arc, thread};

use anyhow::Result;
use config::Config;
use frame_processor::FrameProcessor;
use rccn_usr::transport::{
    RxTransport::{self},
    TransportHandler, TransportManager, TxTransport,
};

mod config;
mod frame_processor;

#[tokio::main]
async fn main() -> Result<()> {
    let config_path = Config::find_config_file()?;
    println!("Using config file: {}", config_path.display());
    let config = Arc::new(Config::from_file(config_path)?);
    println!("Loaded configuration: {:#?}", config);

    // Create transport manager and configure virtual channels
    let mut transport_manager = TransportManager::new().await?;

    // Create channel for communication between the bytes-in
    // frame link and the frame processing task.
    let (bytes_in_tx, bytes_in_rx) = bounded(32);

    // Create input transport for the frames-in link
    match &config.frames.r#in.transport {
        RxTransport::Udp(udp_rx_transport) => {
            let addr = udp_rx_transport.listen.clone().parse()?;
            transport_manager
                .udp_handler
                .add_transport_reader(bytes_in_tx, addr)
                .await?;
        }
        _ => todo!(),
    };

    // Create channel for communication between the frames-out
    // task and the bytes-out transport
    let (bytes_out_tx, bytes_out_rx) = bounded(32);

    // Create input transport for the frames-in link
    match &config.frames.out.transport {
        TxTransport::Udp(udp_tx_transport) => {
            let addr = udp_tx_transport.send.clone().parse()?;
            transport_manager
                .udp_handler
                .add_transport_writer(bytes_out_rx, addr)
                .await?;
        }
        _ => todo!(),
    };
    
    // Configure all virtual channels
    for vc in config.virtual_channels.iter() {
        transport_manager.add_virtual_channel(vc).await?;
    }

    // Get the virtual channel maps for frame processing
    //let ((vc_tx_map, vc_rx_map), _transport_handles) = transport_manager.run();

    // Create frame processor and spawn processing threads
    let processor = FrameProcessor::new(config);
    let p_in = processor.clone();
    let p_out = processor.clone();

    let tx_map = transport_manager.tx_map().clone();
    let frame_process_handle =
        thread::spawn(move || p_in.process_incoming_frames(bytes_in_rx, &tx_map));

    let rx_map = transport_manager.rx_map().clone();
    let _frames_out_handle = thread::spawn(move || p_out.process_frames_out(bytes_out_tx, &rx_map));

    // Wait for threads to complete
    if let Err(e) = frame_process_handle.join() {
        println!("Frame processing thread panicked: {:?}", e);
    }

    Ok(())
}
