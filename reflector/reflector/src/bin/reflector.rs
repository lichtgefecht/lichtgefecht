use std::sync::Arc;

use env_logger::Env;
use log::{error, info};
use reflector;
use reflector::{transport::UdpTransport, duplex_pair};
use reflector_core::{Stoppable, Transport};
use reflector_core::Core;
use tokio::signal;

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();


    let (duplex_for_core,duplex_for_transport) = duplex_pair();
    let transport =  Arc::new(UdpTransport::new("reflector".into(), duplex_for_transport));

    let mut core = Core::new(duplex_for_core, transport.clone());
    let core_hook = core.get_shutdown_hook();

    let core_thread = std::thread::spawn(move || core.run() );    


    add_int_hooks(vec![core_hook, transport.clone()]);

    let _ = transport.run().await;
    core_thread.join().unwrap();
}

fn add_int_hooks(hooks: Vec<Arc<dyn Stoppable + Send + Sync>>) {
    tokio::spawn(async move {
        match signal::ctrl_c().await {
            Ok(_) => {
                info!("Received ctrl_c, shutting down reflector");
                hooks.iter()
                    .for_each(|hook| hook.stop());            
            }
            Err(e) => error!("Error waiting for ctrl_c: {}", e),
        }
    });
}

