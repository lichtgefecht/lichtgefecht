use std::{
    cell::RefCell, collections::HashMap, error::Error, net::{Ipv4Addr, SocketAddr, SocketAddrV4}, sync::{atomic::AtomicBool, Arc}, time::Duration
};

use reflector_core::{MsgWithTarget, Stoppable, Transport};
use bytes::Bytes;
use log::{debug, error, info, warn};
use reflector_core::Core;
use tokio::{
    net::UdpSocket,
    select,
    sync::{mpsc, Mutex, Notify, RwLock},
};

use prost::Message;
use reflector_api::lg::{self, broadcast::ReflectorAddr, msg, Broadcast, DeviceType, Msg};

use crate::TokioDuplex;

pub struct UdpTransport {
    hid: String,
    shutdown_notify: Arc<Notify>,
    shutting_down: AtomicBool,
    duplex: Mutex<Option<TokioDuplex<Msg, MsgWithTarget>>>,
    transport_mapping: RwLock<HashMap<u32, u32>>,
}

const BCA: SocketAddr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(255, 255, 255, 255), 3333));

struct Frame(SocketAddr, Bytes); // TODO should have protocol agnostic target `hid` instead of socket addr

impl Transport for UdpTransport {
    async fn run(&self) -> Result<(), Box<dyn Error>> {
        info!("Starting UdpTransport");

        let socket = UdpSocket::bind(&"0.0.0.0:3333").await?;
        info!("UdpTransport listening on: {}", socket.local_addr()?);
        socket.set_broadcast(true).expect("Kaboom");

        let (snd_tx, mut snd_rx) = mpsc::channel(512);
        let (rcv_tx, mut rcv_rx) = mpsc::channel(512);

        spawn_broadcast_task(snd_tx.clone(), self.hid.clone(), Ipv4Addr::new(192, 168, 0, 146), 3333);

        let duplex = self.duplex.lock().await.take().unwrap();
        let (ctx, crx) = duplex.crack();
        forward_messages_to_transport(snd_tx.clone(), crx);
        forward_messages_to_core(ctx, rcv_rx );

        let mut receive_buffer = vec![0; 1024];

        let notif = self.shutdown_notify.clone();
        loop {
            // either receive a packet or receive the shutdown notification
            select! {
                frame = snd_rx.recv() =>{
                    match frame{
                        Some(frame) => self.handle_send(frame, &socket).await,
                        None => todo!(),
                    }
                }
                rcv = socket.recv_from(&mut receive_buffer) =>{
                    match rcv{
                        Ok(rcv) =>  self.handle_recv_buffer(&rcv_tx, rcv, &receive_buffer).await,
                        Err(e) => error!("rcv error: {e}")
                    }
                }
                _ = notif.notified() =>{
                    warn!("Shutdown notification");
                    self.shutting_down.store(true,std::sync::atomic::Ordering::Relaxed);
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

fn forward_messages_to_transport(sender: mpsc::Sender<Frame>, mut receiver: mpsc::Receiver<MsgWithTarget>) {
    tokio::spawn(async move {
        loop {
            match receiver.recv().await {
                Some(res) => {
                    let mut buf = Vec::with_capacity(res.msg.encoded_len());
                    res.msg.encode(&mut buf).expect("Kaboom");
                    let frame = Frame(BCA, Bytes::copy_from_slice(&buf));
                    //todo: mapping from hid to udp addr
                    match sender.send(frame).await {
                        Ok(_) => (),
                        Err(_e) => {
                            warn!("Failed to send to udp transport");
                        },
                    }      
                    info!("Forwarded message to transport MsgWithTarget -> Frame")
                },
                None => break,
            } 
        }

    });
}
fn forward_messages_to_core(sender: mpsc::Sender<Msg>, mut receiver: mpsc::Receiver<Msg>) {
    tokio::spawn(async move {
        loop {
            match receiver.recv().await {
                Some(res) => {
                    sender.send(res).await.unwrap()
                },
                None => break,
            } 
            info!("Forwarded message to core Msg -> Msg")
        }

    });
}

fn spawn_broadcast_task(tx: mpsc::Sender<Frame>, hid: String, ip: Ipv4Addr, port: u32) {
    tokio::spawn(async move {
        let socket_addr = lg::SocketAddr {
            ip: Some(lg::socket_addr::Ip::V4(ip.into())),
            port,
        };

        let bc = Broadcast {
            device_type: DeviceType::Reflector as i32,
            reflector_addr: Some(ReflectorAddr::SocketAddr(socket_addr)),
        };

        let msg = Msg {
            hid: hid.clone(),
            inner: Some(msg::Inner::Broadcast(bc)),
        };

        let mut buf = Vec::with_capacity(msg.encoded_len());
        msg.encode(&mut buf).expect("Kaboom");
        let bytes = Bytes::copy_from_slice(&buf);
        info!("Will announce my presence: {hid}");
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
            // clone on bytes only increments an Arc internally
            match tx.send(Frame(BCA, bytes.clone())).await {
                Ok(_) => (),
                Err(_e) => {
                    info!("Broadcast channel closed");
                    return;
                },
            }
        }
    });
}

impl Stoppable for UdpTransport {
    fn stop(&self) {
        self.shutting_down
            .store(true, std::sync::atomic::Ordering::Relaxed);
        self.shutdown_notify.clone().notify_waiters();
        info!("Stopping UdpTransport")
    }
}

impl UdpTransport {

    pub fn new(hid: String, duplex: TokioDuplex<Msg, MsgWithTarget>) -> Self {
        UdpTransport {
            shutdown_notify: Arc::new(Notify::new()),
            shutting_down: AtomicBool::new(false),
            duplex: Mutex::new(Some(duplex)),
            hid,
            transport_mapping: RwLock::new(HashMap::new())
        }
    }


    async fn handle_recv_buffer(&self, tx: &mpsc::Sender<Msg>, rcv: (usize, SocketAddr), buf: &[u8]) {
        let (size, _peer) = rcv;

        let buf = &buf[0..size];
        // let mut core = self.core.write().await;

        match Msg::decode(buf) {
            Ok(msg) => {
                match tx.send(msg).await{
                    Ok(_) => (),
                    Err(_) => {
                        info!("rcv_to_core channel closed");
                        return;
                    },
                }
                // core.on_message_received(msg);
            }
            Err(e) => {
                warn!("Invalid msg received: {e}");
                warn!("buf: {buf:?}");
            }
        }
    }

    async fn handle_send(&self, frame: Frame, socket: &UdpSocket) {
        match socket.send_to(&frame.1, frame.0).await {
            Ok(result) => info!("sent {result} bytes"),
            Err(e) => error!("Send error: {e}"),
        }
    }
}

struct TokioDuplexPairForTransport{
    tx: tokio::sync::mpsc::Sender<Msg>,
    rx: tokio::sync::mpsc::Receiver<MsgWithTarget>,
}
struct TokioDuplexPairForTransportTransciever{
    ttx: tokio::sync::mpsc::Sender<MsgWithTarget>,
    trx: tokio::sync::mpsc::Receiver<Msg>,

    stx: std::sync::mpsc::Sender<Msg>,
    srx: std::sync::mpsc::Receiver<MsgWithTarget>,
}

fn switchboard() -> TokioDuplexPairForTransport{
    TokioDuplexPairForTransport {
        tx: tokio::sync::mpsc::channel(1024).0,
        rx: tokio::sync::mpsc::channel(1024).1,
    }
}
