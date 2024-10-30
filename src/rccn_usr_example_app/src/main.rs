use anyhow::Result;
use rccn_usr::{
    config::VirtualChannel,
    service::{AcceptanceError, PusService},
    transport::{config::Ros2RxTransport, RxTransport, TransportManager, TxTransport},
};
use service::ExampleService;

mod command;
mod service;

fn main() -> Result<()> {
    let mut transport_manager = TransportManager::new("rccn_usr_example_app".into())?;

    // Configure virtual channel for TC/TM
    let vc = VirtualChannel {
        id: 0,
        name: "bus_realtime".into(),
        splitter: None,
        tx_transport: Some(TxTransport::Ros2("/vc/bus_realtime/tx".into())),
        rx_transport: Some(RxTransport::Ros2(Ros2RxTransport::with_topic(
            "/vc/bus_realtime/rx",
        ))),
    };
    transport_manager.add_virtual_channel(&vc)?;

    // Create our ExampleService
    let mut example_service = ExampleService::new(42, transport_manager.get_vc_maps().0);

    // Start the transport manager
    let ((_, mut vc_rx_map), transport_handles) = transport_manager.run();

    // Process incoming TCs from the virtual channel
    let tc_receiver = vc_rx_map.remove(&0).expect("VC 0 TC receiver not found");
    loop {
        match tc_receiver.recv() {
            Ok(bytes) => match example_service.handle_tc_bytes(&bytes) {
                Ok(()) => {
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
