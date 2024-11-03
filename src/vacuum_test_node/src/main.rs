use anyhow::Result;
use rccn_usr::{
    config::VirtualChannel,
    service::{AcceptanceError, PusService},
    transport::{ros2::new_shared_ros2_node, TransportManager},
};
use stress_service::service::StressTestService;

mod stress_service;

fn main() -> Result<()> {
    let node = new_shared_ros2_node("vacuum_test_node", &"/")?;

    let mut transport_manager = TransportManager::new_with_ros2_node(node.clone())?;
    transport_manager.add_virtual_channel(&VirtualChannel::new_ros2("bus_realtime", 0))?;
    let ((vc_tx_map, mut vc_rx_map), transport_handles) = transport_manager.run();

    let mut stress_service = StressTestService::new(42, &vc_tx_map, node.clone());

    // Process incoming TCs from the virtual channel
    let tc_receiver = vc_rx_map.remove(&0).expect("VC 0 TC receiver not found");
    loop {
        match tc_receiver.recv() {
            Ok(bytes) => match stress_service.handle_tc_bytes(&bytes) {
                Ok(_) => {
                    println!("Command handled succesfully.");
                }
                Err(AcceptanceError::UnknownApid(apid)) => {
                    println!("Command was for APID {apid}, ignoring.");
                }
                Err(e) => {
                    println!("Error handling command: {e:?}");
                }
            },
            Err(e) => {
                println!("TC RX channel closed, exiting. {e:?}");
                break;
            }
        }
    }

    // Wait for transport threads to complete
    for handle in transport_handles {
        handle.join().unwrap()?;
    }

    println!("Closing down.");
    Ok(())
}
