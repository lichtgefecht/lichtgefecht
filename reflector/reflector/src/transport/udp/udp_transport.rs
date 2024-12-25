use std::{
    error::Error,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    sync::{atomic::AtomicBool, Arc},
    time::Duration,
};

use crate::transport::{Stoppable, Transport};
use bytes::Bytes;
use log::{error, info, warn};
use reflector_core::Core;
use tokio::{
    net::UdpSocket,
    select,
    sync::{mpsc, Notify},
};

pub struct UdpTransport {
    shutdown_notify: Arc<Notify>,
    shutting_down: AtomicBool,
    core: Core,
}
impl UdpTransport {
    pub fn new(core: Core) -> Self {
        UdpTransport {
            shutdown_notify: Arc::new(Notify::new()),
            shutting_down: AtomicBool::new(false),
            core,
        }
    }
}

const BCA: SocketAddr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(255, 255, 255, 255), 3333));

struct Frame(SocketAddr, Bytes);

impl Transport for UdpTransport {
    async fn run(&self) -> Result<(), Box<dyn Error>> {
        info!("Starting UdpTransport");

        let socket = UdpSocket::bind(&"192.168.0.146:3333").await?;
        info!("UdpTransport listening on: {}", socket.local_addr()?);
        socket.set_broadcast(true).expect("Kaboom");

        let (tx, mut rx) = mpsc::channel(512);

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(1)).await;
                info!("Announcing my presence");
                tx.send(Frame(
                    BCA,
                    Bytes::copy_from_slice("test".as_bytes()),
                ))
                .await
                .expect("Kaboom");
            }
        });

        let mut buf = vec![0; 1024];
        // let mut to_send = None;

        let notif = self.shutdown_notify.clone();
        loop {
            // either receive a packet or receive the shutdown notification
            select! {
                frame = rx.recv() =>{
                    match frame{
                        Some(frame) => self.handle_send(frame, &socket).await,
                        None => todo!(),
                    }
                }
                rcv = socket.recv_from(&mut buf) =>{
                    match rcv{
                        Ok(rcv) =>  self.handle_recv_buffer(rcv, &buf),
                        Err(e) => error!("rcv error: {e}")
                    }
                }
                _ = notif.notified() =>{
                    warn!("Shutdown notification");
                    return Ok(());
                }
            };

            // if select is not fair, we might starve the shutdown notifications
            // therefore, the shutting_down boolean is checked after each receive
            if self
                .shutting_down
                .load(std::sync::atomic::Ordering::Relaxed)
            {
                return Ok(());
            }
        }
    }
}

impl Stoppable for UdpTransport{
    fn stop(&self) {
        self.shutting_down
            .store(true, std::sync::atomic::Ordering::Relaxed);
        self.shutdown_notify.clone().notify_waiters();
        info!("Stopping UdpTransport")
    }
}


impl UdpTransport {
    fn handle_recv_buffer(&self, rcv: (usize, SocketAddr), _buf: &[u8]) {
        let (size, peer) = rcv;
        // if peer.ip() ==  {
        //     info!("Ignoring broadcast")
        // }
        info!("[{}] rcv {} bytes", peer, size);
        self.core.on_message_received(); // TODO pass in things
    }
    async fn handle_send(&self, frame: Frame, socket: &UdpSocket) {
        match socket.send_to(&frame.1, frame.0).await {
            Ok(result) => info!("sent {result} bytes"),
            Err(e) => error!("Send error: {e}"),
        }
    }
}
