use std::os::unix::thread;
use std::sync::Arc;

use env_logger::Env;
use log::{error, info};
use reflector;
use reflector::transport::UdpTransport;
use reflector_core::{Stoppable, Transport};
use reflector_core::Core;
use tokio::signal;

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();



    let (ctt_tx,ctt_rx) = tokio::sync::mpsc::channel(512);
    let (ttc_tx,ttc_rx) = tokio::sync::mpsc::channel(512);

    let mut core = Core::new(ctt_tx, ttc_rx);

    std::thread::spawn(move || {
        core.run();
    });    

    let transport = UdpTransport::new("reflector".into(), ttc_tx);
    let transport = Arc::new(transport);

    add_int_hook(transport.clone());
    let _ = transport.run(ctt_rx).await;
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
