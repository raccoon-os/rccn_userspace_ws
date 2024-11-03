use crossbeam_channel::bounded;
use std::{sync::Arc, thread};

use config::Config;
use frame_processor::FrameProcessor;
use rccn_usr::transport::{
    RxTransport::{self},
    TransportManager, TxTransport,
};

mod config;
mod frame_processor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config_path = Config::find_config_file()?;
    println!("Using config file: {}", config_path.display());
    let config = Arc::new(Config::from_file(config_path)?);
    println!("Loaded configuration: {:#?}", config);

    // Create transport manager and configure virtual channels
    let mut transport_manager = TransportManager::new("rccn_usr_comm".into())?;

    // Create channel for communication between the bytes-in
    // frame link and the frame processing task.
    let (bytes_in_tx, bytes_in_rx) = bounded(32);

    // Create input transport for the frames-in link
    match &config.frames.r#in.transport {
        RxTransport::Udp(udp_rx_transport) => {
            let addr = udp_rx_transport.listen.clone().parse()?;
            transport_manager.add_udp_reader(bytes_in_tx, addr);
        }
        RxTransport::Ros2(ros2_rx_transport) => todo!(),
    };

    // Create channel for communication between the frames-out
    // task and the bytes-out transport
    let (bytes_out_tx, bytes_out_rx) = bounded(32);

    // Create input transport for the frames-in link
    match &config.frames.out.transport {
        TxTransport::Udp(udp_tx_transport) => {
            let addr = udp_tx_transport.send.clone().parse()?;
            transport_manager.add_udp_writer(bytes_out_rx, addr);
        }
        TxTransport::Ros2(_) => {
            todo!();
        }
    };

    // Configure all virtual channels
    for vc in config.virtual_channels.iter() {
        transport_manager.add_virtual_channel(vc)?;
    }

    // Get the virtual channel maps for frame processing
    let ((vc_tx_map, vc_rx_map), _transport_handles) = transport_manager.run();

    // Create frame processor and spawn processing threads
    let processor = FrameProcessor::new(config);
    let p_in = processor.clone();
    let p_out = processor.clone();

    let frame_process_handle =
        thread::spawn(move || p_in.process_incoming_frames(bytes_in_rx, &vc_tx_map));

    let _frames_out_handle =
        thread::spawn(move || p_out.process_frames_out(bytes_out_tx, &vc_rx_map));

    // Wait for threads to complete
    if let Err(e) = frame_process_handle.join() {
        println!("Frame processing thread panicked: {:?}", e);
    }
    Ok(())
}
