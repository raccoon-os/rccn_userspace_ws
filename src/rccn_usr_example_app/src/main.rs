use std::thread;

use crossbeam_channel::{bounded, Sender};
use rccn_usr::transport::{
    ros2::{Ros2ReaderConfig, Ros2TransportHandler},
    TransportHandler,
};
use spacepackets::{ecss::tc::PusTcReader, PacketId, PacketSequenceCtrl, SpHeader};

fn handle_tc_bytes(bytes: Vec<u8>, tm_tx: Sender<Vec<u8>>) {
    match PusTcReader::new(&bytes) {
        Ok((tc, packet_size)) => {
            println!(
                "Got PUS TC: {tc:?}, packet size {packet_size}, data {:?}",
                tc.app_data()
            );

            println!("Sending some fake telemetry back");

            let fake_tm_header = SpHeader::new(
                PacketId::new(spacepackets::PacketType::Tm, false, 1),
                PacketSequenceCtrl::new(spacepackets::SequenceFlags::Unsegmented, 1234),
                2,
            );
            let tm_data = [0x13, 0x37];

            tm_tx.send(Vec::from(fake_tm_header.to_vec().concat(tm_data)));
        }
        Err(e) => {
            println!("Error getting source packet header: {e:?}");
        }
    }
}
fn main() {
    let tc_rx_topic = "/vc/bus_realtime/rx";
    let tm_tx_topic = "/vc/bus_realtime/tx";

    let (tc_in_tx, tc_in_rx) = bounded(32);
    let (tm_out_tx, tm_out_rx) = bounded(32);

    let mut ros2_handler = Ros2TransportHandler::new("rccn_usr_example_app".into()).unwrap();
    ros2_handler.add_transport_reader(tc_in_tx, Ros2ReaderConfig::Subscription(tc_rx_topic.into()));
    ros2_handler.add_transport_writer(tm_out_rx, tm_tx_topic.into());
    thread::spawn(move || ros2_handler.run());

    loop {
        match tc_in_rx.recv() {
            Ok(bytes) => {
                handle_tc_bytes(bytes, tm_out_tx.clone());
            }
            Err(e) => {
                println!("Error receiving from TC in channel. Exiting. {e:?}");
                break;
            }
        }
    }

    println!("Closing down.")
}
