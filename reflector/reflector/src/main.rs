use std::sync::Arc;

use log::{error, info};
use tokio::signal;
use transport::{Transport, UdpTransport};

mod transport;

#[tokio::main]
async fn main() {
    env_logger::init();

    let transport = UdpTransport::new();
    let transport = Arc::new(transport);

    let tc = transport.clone();
    tokio::spawn(async move {
        let cc = signal::ctrl_c().await;
        match cc {
            Ok(_) => {
                info!("Received ctrl_c, shutting down reflector");
                tc.stop();
            }
            Err(e) => error!("Error waiting for ctrl_c: {}", e),
        }
    });

    let _ = transport.run().await;
}
