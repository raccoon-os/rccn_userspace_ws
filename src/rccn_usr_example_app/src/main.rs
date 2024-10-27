use futures::{executor::LocalPool, task::{LocalSpawnExt, SpawnExt}, StreamExt};
use rccn_usr::r2r::{self, rccn_usr_msgs::msg::RawBytes, QosProfile};

fn main() -> anyhow::Result<()> {
    let mut pool = LocalPool::new();
    let spawner = pool.spawner();

    let ctx = r2r::Context::create()?;
    let mut node = r2r::Node::create(ctx, "rccn_usr_example_app", "/")?;

    let mut sub = node.subscribe::<RawBytes>("/vc/bus_realtime/rx", QosProfile::default()).unwrap();

    spawner.spawn(async move {
        loop {
            match sub.next().await {
                Some(msg) => {
                    println!("Got message {:?}", msg);
                }
                None => {
                    println!("Subscription closed, exiting.");
                    break;
                }
            }
        }
    })?;

    loop {
        node.spin_once(std::time::Duration::from_millis(100));
        pool.run_until_stalled();
    }

    Ok(())
}
