use std::sync::Arc;

use env_logger::Env;
use log::{error, info};
use reflector_core::Core;
use tokio::signal;
use transport::{Stoppable, SyncTransport, Transport, UdpTransport2};

mod transport;

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("warn")).init();

    let core = Core::new();

    let transport = UdpTransport2::new();
    // let transport = UdpTransport::new(core);
    let transport = Arc::new(transport);

    // add_int_hook(transport.clone());
    let _ = transport.run();
}

fn add_int_hook(tc: Arc<dyn Stoppable + Send + Sync>)  {
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
