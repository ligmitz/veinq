mod server;
mod storage;

use std::path::Path;
use std::sync::{Arc, Mutex};
use storage::Broker;

#[tokio::main]
async fn main() {
    let mut broker = Broker::new(Path::new("logs")).expect("Failed to initialize broker");
    broker.create_topic("orders", 3).ok();
    broker.create_topic("payments", 2).ok();
    let broker = Arc::new(Mutex::new(broker));
    server::run(broker).await;
}
