use anyhow::Result;
use example_service::service::ExampleService;
use rccn_usr::{config::VirtualChannel, transport::ros2::new_shared_ros2_node};
use rccn_usr_pus::app::PusApp;

mod example_service;

const APID: u16 = 42;

fn main() -> Result<()> {
    let node = new_shared_ros2_node("rccn_usr_example_app", &"/")?;
    let mut app = PusApp::new_with_ros2_node(APID, node.clone());

    app.add_virtual_channel(&VirtualChannel::on_ros2_topic(0, "bus_realtime"))?;

    let service = ExampleService::new();
    app.register_service(service);

    app.run();
    Ok(())
}
