use std::sync::Arc;

use env_logger::Env;
use log::{error, info};
use reflector::{config::read_config, tokio_tools::duplex_pair, transport::UdpTransport};
use reflector_core::{api::infra::Stoppable, api::transport::Transport, Core};
use tokio::signal;

#[tokio::main]
async fn main() {
    let config = read_config();

    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    info!("{config:#?}");

    let (duplex_for_core, duplex_for_transport) = duplex_pair();
    let transport = Arc::new(UdpTransport::new(config.transport, duplex_for_transport));

    
    let core_thread = std::thread::spawn(move || {
        let mut core = Core::new(duplex_for_core);
        core.run()
    });

    add_shutdown_hooks(vec![transport.clone()]);

    let _ = transport.run().await;
    core_thread.join().unwrap();
}

fn add_shutdown_hooks(hooks: Vec<Arc<dyn Stoppable + Send + Sync>>) {
    tokio::spawn(async move {
        match signal::ctrl_c().await {
            Ok(_) => {
                info!("Received ctrl_c, shutting down reflector");
                hooks.iter().for_each(|hook| hook.stop());
            }
            Err(e) => error!("Error waiting for ctrl_c: {}", e),
        }
    });
}
