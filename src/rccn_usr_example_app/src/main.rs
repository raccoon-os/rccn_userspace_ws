use std::thread;
use rccn_usr::{
    config::VirtualChannel,
    transport::{config::{Ros2RxTransport, Ros2TxTransport}, RxTransport, TransportManager, TxTransport},
};
use spacepackets::{
    ecss::{
        tc::PusTcReader,
        tm::{PusTmCreator, PusTmSecondaryHeader}, WritablePusPacket,
    },
    PacketId, PacketSequenceCtrl, PacketType, SequenceFlags, SpHeader,
};
use anyhow::Result;

mod app;

fn main() -> Result<()> {
    let mut transport_manager = TransportManager::new("rccn_usr_example_app".into())?;

    // Configure virtual channel for TC/TM
    let vc = VirtualChannel {
        id: 0,
        name: "bus_realtime".into(),
        splitter: None,
        tx_transport: Some(TxTransport::Ros2(Ros2TxTransport {
            topic_pub: "/vc/bus_realtime/rx".into(),
        })),
        rx_transport: Some(RxTransport::Ros2(Ros2RxTransport {
            topic_sub: Some("/vc/bus_realtime/tx".into()),
            action_srv: None,
        })),
    };

    transport_manager.add_virtual_channel(&vc)?;
    //let (vc_in_map, vc_out_map) = transport_manager.get_vc_maps();
    let ((vc_tx_map, mut vc_rx_map), transport_handles) = transport_manager.run();

    // Get the sender for our virtual channel
    let tm_sender = vc_tx_map.get(&0).expect("VC 0 not found");

    // Process incoming TCs from the virtual channel
    if let Some(tc_receiver) = vc_rx_map.remove(&0) {
        loop {
            match tc_receiver.recv() {
                Ok(bytes) => {
                    match PusTcReader::new(&bytes) {
                        Ok((tc, packet_size)) => {
                            println!(
                                "Got PUS TC: {tc:?}, packet size {packet_size}, data {:?}",
                                tc.app_data()
                            );

                            let fake_tm_header = SpHeader::new(
                                PacketId::new(PacketType::Tm, false, 1),
                                PacketSequenceCtrl::new(SequenceFlags::Unsegmented, 1234),
                                2,
                            );

                            let payload = [0x13, 0x37];
                            let timestamp = [0u8; 7];
                            let tm = PusTmCreator::new(
                                fake_tm_header,
                                PusTmSecondaryHeader::new(130, 1, 0, 0, &timestamp),
                                &payload,
                                true,
                            );

                            println!("Sending some fake telemetry back");
                            if let Err(e) = tm_sender.send(tm.to_vec().unwrap()) {
                                println!("Error sending TM: {e:?}");
                                break;
                            }
                        }
                        Err(e) => {
                            println!("Error getting source packet header: {e:?}");
                        }
                    }
                }
                Err(e) => {
                    println!("Error receiving from TC channel. Exiting. {e:?}");
                    break;
                }
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
