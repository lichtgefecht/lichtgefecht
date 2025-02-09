use std::sync::Arc;

use env_logger::Env;
use log::{error, info};
use reflector;
use reflector::transport::{Stoppable, Transport, UdpTransport};
use reflector_core::Core;
use tokio::signal;

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();

    let core = Core::new();

    let transport = UdpTransport::new(core, "test".into());
    let transport = Arc::new(transport);

    // add_int_hook(transport.clone());
    let _ = transport.run().await;
}

fn add_int_hook(tc: Arc<dyn Stoppable + Send + Sync>) {
    tokio::spawn(async move {
        match signal::ctrl_c().await {
            Ok(_) => {
                info!("Received ctrl_c, shutting down reflector");
                tc.stop();
            }
            Err(e) => error!("Error waiting for ctrl_c: {}", e),
        }
    });
}
