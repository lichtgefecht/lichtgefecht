use std::{
    error::Error,
    sync::{atomic::AtomicBool, Arc},
};

use crate::transport::Transport;
use log::{error, info, warn};
use tokio::{net::UdpSocket, select, sync::Notify};
pub struct UdpTransport {
    shutdown_notify: Arc<Notify>,
    shutting_down: AtomicBool,
}
impl UdpTransport {
    pub fn new() -> Self {
        UdpTransport {
            shutdown_notify: Arc::new(Notify::new()),
            shutting_down: AtomicBool::new(false),
        }
    }
}

impl Transport for UdpTransport {
    async fn run(&self) -> Result<(), Box<dyn Error>> {
        info!("Starting UdpTransport");

        let socket = UdpSocket::bind(&"0.0.0.0:3333").await?;
        info!("UdpTransport listening on: {}", socket.local_addr()?);

        let mut buf = vec![0; 1024];
        let mut to_send = None;

        let notif = self.shutdown_notify.clone();
        loop {
            if let Some((size, peer)) = to_send {
                let _amt = socket.send_to(&buf[..size], &peer).await?;
                // println!("Echoed {amt}/{size} bytes to {peer}");
                info!("[{peer}] sent {size} bytes");
            }

            // either receive a packet or receive the shutdown notification
            select! {
                val = socket.recv_from(&mut buf) =>{
                    if let Ok((size, peer)) = val{
                        info!("[{}] rcv {} bytes", peer, size);
                        to_send = Some((size, peer));
                    }
                    else{
                        error!("rcv error");
                    }
                }
                _ = notif.notified() =>{
                    warn!("Shutdown notification");
                    return Ok(());
                }
            };

            // if select is not fair, we maight starve the shutdown notifications
            // therefore, the shutting_down boolean is checked after each receive
            if self
                .shutting_down
                .load(std::sync::atomic::Ordering::Relaxed)
            {
                return Ok(());
            }
        }
    }

    fn stop(&self) {
        self.shutting_down
            .store(true, std::sync::atomic::Ordering::Relaxed);
        self.shutdown_notify.clone().notify_waiters();
        info!("Stopping UdpTransport")
    }
}
