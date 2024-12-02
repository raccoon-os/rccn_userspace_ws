use anyhow::Result;
use example_service::service::ExampleService;
use rccn_usr::config::VirtualChannel;
use rccn_usr_pus::app::PusApp;

mod example_service;

const APID: u16 = 42;

#[tokio::main]
async fn main() -> Result<()> {
    let mut app = PusApp::new(APID).await;

    app.add_virtual_channel(&VirtualChannel::on_z_topic(0, "bus_realtime").unwrap()).await?;

    let service = ExampleService::new();
    app.register_service(service);

    app.run();
    Ok(())
}
