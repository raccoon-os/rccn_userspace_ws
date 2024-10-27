use std::thread;

use crossbeam_channel::{bounded, Sender};
use rccn_usr::transport::{
    ros2::{Ros2ReaderConfig, Ros2TransportHandler},
    TransportHandler,
};
use spacepackets::{
    ecss::{
        tc::PusTcReader,
        tm::{PusTmCreator, PusTmSecondaryHeader}, WritablePusPacket,
    },
    PacketId, PacketSequenceCtrl, PacketType, SequenceFlags, SpHeader,
};

fn handle_tc_bytes(bytes: Vec<u8>, tm_tx: Sender<Vec<u8>>) {
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
            let _ = tm_tx.send(tm.to_vec().unwrap());
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
