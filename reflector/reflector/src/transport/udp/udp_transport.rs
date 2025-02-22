use std::{
    collections::HashMap,
    error::Error,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4},
    sync::{atomic::AtomicBool, Arc},
    time::Duration,
};

use bytes::Bytes;
use log::{debug, error, info, warn};
use reflector_core::{MsgWithTarget, Stoppable, Transport, TransportHandle};
use tokio::{
    net::UdpSocket,
    select,
    sync::{mpsc, Mutex, Notify, RwLock},
};

use prost::Message;
use reflector_api::lg::{
    self, broadcast::ReflectorAddr, msg, socket_addr::Ip, Broadcast, DeviceType, Msg,
};

use crate::tokio_tools::TokioDuplex;

pub struct UdpTransport {
    hid: String,
    shutdown_notify: Arc<Notify>,
    shutting_down: AtomicBool,
    duplex: Mutex<Option<TokioDuplex<Msg, MsgWithTarget>>>,
    transport_mapping: TransportMap,
}

const BCA: SocketAddr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(255, 255, 255, 255), 3333));
type TransportMap = Arc<RwLock<HashMap<String, SocketAddr>>>;
struct Frame(SocketAddr, Bytes);

impl Transport for UdpTransport {
    async fn run(&self) -> Result<(), Box<dyn Error>> {
        info!("Starting UdpTransport");

        let socket = UdpSocket::bind(&"0.0.0.0:3333").await?;
        info!("UdpTransport listening on: {}", socket.local_addr()?);
        socket.set_broadcast(true).expect("Kaboom");

        let (snd_tx, mut snd_rx) = mpsc::channel(512);
        let (core_tx, core_rx) = self.crack_duplex().await;

        let notif = self.shutdown_notify.clone();

        spawn_broadcast_task(
            snd_tx.clone(),
            self.hid.clone(),
            Ipv4Addr::new(192, 168, 0, 146),
            3333,
        );
        forward_messages_to_transport(snd_tx.clone(), core_rx, self.transport_mapping.clone());

        let mut receive_buffer = vec![0; 1024];

        loop {
            // either receive a packet, send one or receive the shutdown notification
            let _ = select! {
                frame = snd_rx.recv()                           => self.handle_send(frame, &socket).await,
                rcv = socket.recv_from(&mut receive_buffer)     => self.handle_recv_buffer(&core_tx, rcv, &receive_buffer).await,
                _ = notif.notified()                            => self.handle_shutdown_notification().await,
            };

            // if select is not fair, we might starve the shutdown notifications
            // therefore, the shutting_down boolean is checked after each receive
            if self
                .shutting_down
                .load(std::sync::atomic::Ordering::Relaxed)
            {
                warn!("Transport shutdown");
                return Ok(());
            }
        }
    }
}

impl UdpTransport {
    pub fn new(hid: String, duplex: TokioDuplex<Msg, MsgWithTarget>) -> Self {
        UdpTransport {
            shutdown_notify: Arc::new(Notify::new()),
            shutting_down: AtomicBool::new(false),
            duplex: Mutex::new(Some(duplex)),
            hid,
            transport_mapping: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn handle_recv_buffer(
        &self,
        tx: &mpsc::Sender<Msg>,
        rcv: Result<(usize, SocketAddr), std::io::Error>,
        buf: &[u8],
    ) -> Result<(), ()> {
        let (size, _peer) = rcv.map_err(|_| ())?;
        let buf = &buf[0..size];
        let msg = Msg::decode(buf).map_err(|e| warn!("Decoding error: {e}, {buf:?}"))?;
        tx.send(msg)
            .await
            .map_err(|_| warn!("rcv_to_core channel closed"))
    }

    async fn handle_send(&self, frame: Option<Frame>, socket: &UdpSocket) -> Result<(), ()> {
        let Frame(addr, buf) = frame.ok_or(())?;
        let result = socket
            .send_to(&buf, addr)
            .await
            .map_err(|e| warn!("Send err: {e}"))?;
        debug!("sent {result} bytes");
        Ok(())
    }

    async fn handle_shutdown_notification(&self) -> Result<(), ()> {
        self.shutting_down
            .store(true, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }

    async fn crack_duplex(&self) -> (mpsc::Sender<Msg>, mpsc::Receiver<MsgWithTarget>) {
        let duplex = self.duplex.lock().await.take().unwrap();
        let (ctx, crx) = duplex.crack();
        (ctx, crx)
    }
}

impl TransportHandle for UdpTransport {
    fn add_address_entry(&self, hid: String, addr: lg::broadcast_reply::ClientAddr) {
        match addr {
            lg::broadcast_reply::ClientAddr::SocketAddr(socket_addr) => {
                let addr = match socket_addr.ip {
                    Some(Ip::V4(ip)) => SocketAddr::new(
                        IpAddr::V4(Ipv4Addr::from_bits(ip)),
                        socket_addr.port as u16,
                    ),
                    Some(Ip::V6(_)) => {
                        SocketAddr::new(IpAddr::V6(Ipv6Addr::from_bits(0)), socket_addr.port as u16)
                    }
                    None => todo!(),
                };
                self.transport_mapping.blocking_write().insert(hid, addr);
            }
            _ => error!("UdpTransport does not support address types other than SocketAddr"),
        }
    }
}

fn forward_messages_to_transport(
    sender: mpsc::Sender<Frame>,
    mut receiver: mpsc::Receiver<MsgWithTarget>,
    transport_mapping: TransportMap,
) {
    tokio::spawn(async move {
        while let Some(msg) = receiver.recv().await {
            let mut buf = Vec::with_capacity(msg.msg.encoded_len());
            msg.msg.encode(&mut buf).expect("Kaboom");
            if let Some(addr) = transport_mapping.read().await.get(&msg.target_hid) {
                let frame = Frame(*addr, Bytes::copy_from_slice(&buf));
                match sender.send(frame).await {
                    Ok(_) => {
                        debug!("Forwarded message to transport MsgWithTarget -> Frame")
                    }
                    Err(_e) => warn!("Failed to send to udp transport"),
                }
            } else {
                warn!("Ignoring send command to unknown hid: {}", msg.target_hid)
            }
        }
        warn!("forward_messages_to_transport task exiting")
    });
}

fn spawn_broadcast_task(tx: mpsc::Sender<Frame>, hid: String, ip: Ipv4Addr, port: u32) {
    tokio::spawn(async move {
        let msg = get_bc_message(&hid, ip, port);
        let mut buf = Vec::with_capacity(msg.encoded_len());
        msg.encode(&mut buf).expect("Kaboom");
        let bytes = Bytes::copy_from_slice(&buf);
        info!("Will announce my presence: {hid}");
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
            if tx.send(Frame(BCA, bytes.clone())).await.is_err() {
                info!("Broadcast channel closed");
                return;
            }
        }
    });
}

fn get_bc_message(hid: &str, ip: Ipv4Addr, port: u32) -> Msg {
    let socket_addr = lg::SocketAddr {
        ip: Some(lg::socket_addr::Ip::V4(ip.into())),
        port,
    };

    let bc = Broadcast {
        device_type: DeviceType::Reflector as i32,
        reflector_addr: Some(ReflectorAddr::SocketAddr(socket_addr)),
    };

    Msg {
        hid: hid.to_owned(),
        inner: Some(msg::Inner::Broadcast(bc)),
    }
}

impl Stoppable for UdpTransport {
    fn stop(&self) {
        self.shutting_down
            .store(true, std::sync::atomic::Ordering::Relaxed);
        self.shutdown_notify.clone().notify_waiters();
    }
}
