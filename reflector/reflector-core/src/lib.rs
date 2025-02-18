use handlers::{BroadcastReplyHandler, IgnoredMessageHandler, UnimplementedMessageHandler};
use reflector_api::lg::{
    self, broadcast::ReflectorAddr, broadcast_reply::ClientAddr, msg, Broadcast, DeviceType, Msg,
};
use tokio::sync::mpsc;
use std::{collections::HashMap, error::Error, fmt::Debug, future::Future};

use log::{info, warn};

mod handlers;

pub struct Game {}

pub struct Team {}

#[derive(Debug)]
pub struct Device {
    pub(crate) hid: String,
    pub(crate) device_type: i32,
    pub(crate) client_addr: ClientAddr,
}

pub struct Player {}

pub struct Binding {}

#[derive(Default, Debug)]
pub struct State {
    devices: HashMap<String, Device>,
}

pub trait Transport: Stoppable {
    fn new(hid: String, rcv_to_core: mpsc::Sender<Msg>) -> Self;
    fn run(&self, core_to_send: mpsc::Receiver<Envelope>) -> impl Future<Output = Result<(), Box<dyn Error>>> + Send;
    fn send(&self);
}

pub trait Stoppable {
    fn stop(&self);
}

pub struct Envelope(pub String, pub Msg);

pub struct Core {
    state: State,
    tx: mpsc::Sender<Envelope>,
    rx: mpsc::Receiver<Msg>,
}

fn to_message_handler<'a>(msg: Msg) -> Box<dyn MessageHandler> {
    match msg {
        Msg {
            inner: Some(msg::Inner::Broadcast(_)),
            ..
        } => IgnoredMessageHandler::new(),
        Msg {
            hid,
            inner: Some(msg::Inner::BroadcastReply(broadcast_reply)),
            ..
        } => BroadcastReplyHandler::new(hid, broadcast_reply),
        i => UnimplementedMessageHandler::new(format!("{i:?}")),
    }
}

pub trait MessageHandler {
    fn handle(&self, core: &mut Core);
}

impl Core {
    pub fn new(tx: mpsc::Sender<Envelope>, rx: mpsc::Receiver<Msg>) -> Self {
        Core {
            state: State::default(),
            tx,
            rx
        }
    }
    pub fn run(&mut self) {
       let msg = self.rx.blocking_recv().unwrap();
       self.on_message_received(msg);
    }

    pub fn on_message_received(&mut self, msg: Msg) {
        let handler = to_message_handler(msg);
        handler.handle(self);
    }
    pub fn shutdown(&mut self) {
        info!("Shutting down core");
        info!("Registered clients: {:?}", self.state.devices);
    }
}
