use std::sync::Arc;

use log::{error, info};
use reflector_core::Core;
use tokio::signal;
use transport::{Transport, UdpTransport};

mod transport;

#[tokio::main]
async fn main() {
    env_logger::init();

    let core = Core::new();

    let transport = UdpTransport::new(core);
    let transport = Arc::new(transport);

    add_int_hook(transport.clone());
    let _ = transport.run().await;
}

fn add_int_hook(tc: Arc<UdpTransport>) {
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
