use anyhow::Result;
use crossbeam_channel::bounded;
use std::{net::SocketAddr, sync::Arc, thread};
use rccn_usr::types::{VirtualChannelInMap, VirtualChannelOutMap};

use config::{Config, InputTransport, OutputTransport};
use frame_processor::FrameProcessor;
use rccn_usr::transport::{TransportManager, ros2::Ros2ReaderConfig};

mod config;
mod frame_processor;

fn main() -> Result<()> {
    let config = Arc::new(Config::from_file("etc/config.yaml")?);
    println!("Loaded configuration: {:#?}", config);

    // Create transport manager
    let mut transport_manager = TransportManager::new("rccn_usr_comm".into())?;

    // Create channel for communication between the bytes-in
    // frame link and the frame processing task.
    let (bytes_in_tx, bytes_in_rx) = bounded(32);

    // Create input transport for the frames-in link
    match &config.frames.r#in.transport {
        InputTransport::Udp(udp_input_transport) => {
            let addr = udp_input_transport.listen.clone().parse()?;
            transport_manager.add_udp_reader(bytes_in_tx, addr);
        }
        InputTransport::Ros2(_) => {
            todo!();
        }
    };

    // Create channel for communication between the frames-out
    // task and the bytes-out transport
    let (bytes_out_tx, _bytes_out_rx) = bounded(32);

    // Keep the virtual channel senders and receivers in a HashMap
    let mut vc_in_map = VirtualChannelInMap::new();
    let mut vc_out_map = VirtualChannelOutMap::new();

    for vc in config.virtual_channels.iter() {
        // Prepare in-direction channels and transport
        let (vc_in_tx, vc_in_rx) = bounded(32);
        let rx_added = match &vc.in_transport {
            Some(InputTransport::Udp(addr)) => {
                let addr: SocketAddr = addr.listen.clone().parse()?;
                transport_manager.add_udp_writer(vc_in_rx, addr);
                true
            }
            Some(InputTransport::Ros2(ros2_transport)) => {
                transport_manager.add_ros2_writer(vc_in_rx, ros2_transport.topic_pub.clone());
                true
            }
            _ => false,
        };
        if rx_added {
            vc_in_map.insert(vc.id, vc_in_tx);
        }

        // Prepare out-direction channels and transport
        let (vc_out_tx, vc_out_rx) = bounded(32);
        let tx_added = match &vc.out_transport {
            Some(OutputTransport::Udp(addr)) => {
                let addr = addr.send.clone().parse()?;
                transport_manager.add_udp_reader(vc_out_tx, addr);
                true
            }
            Some(OutputTransport::Ros2(ros2_transport)) => {
                let reader_config = if let Some(topic) = &ros2_transport.topic_sub {
                    Ros2ReaderConfig::Subscription(topic.clone())
                } else if let Some(action_srv) = &ros2_transport.action_srv {
                    Ros2ReaderConfig::ActionServer(action_srv.clone())
                } else {
                    panic!("Your invalid config has somehow made it here. This should not happen");
                };

                transport_manager.add_ros2_reader(vc_out_tx, reader_config);
                true
            }
            None => false, // No output transport configured.
        };
        if tx_added {
            vc_out_map.insert(vc.id, vc_out_rx);
        }
    }

    let _transport_handles = transport_manager.run();

    // Create frame processor and spawn processing threads
    let processor = FrameProcessor::new(config);
    let p_in = processor.clone();
    let p_out = processor.clone();

    let frame_process_handle =
        thread::spawn(move || p_in.process_incoming_frames(bytes_in_rx, &vc_in_map));

    let _frames_out_handle =
        thread::spawn(move || p_out.process_frames_out(bytes_out_tx, &vc_out_map));

    // Wait for threads to complete
    if let Err(e) = frame_process_handle.join() {
        println!("Frame processing thread panicked: {:?}", e);
    }
    Ok(())
}
