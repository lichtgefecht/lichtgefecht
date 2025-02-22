use handlers::{BroadcastReplyHandler, IgnoredMessageHandler, UnimplementedMessageHandler};
use reflector_api::lg::{
    self, broadcast::ReflectorAddr, broadcast_reply::ClientAddr, msg, Broadcast, DeviceType, Msg,
};
use std::sync::mpsc::{self, RecvError, SendError};
use std::{collections::HashMap, error::Error, fmt::Debug, future::Future, sync::{atomic::{AtomicBool, Ordering}, Arc}};

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


pub trait Duplex<T,R> {
    fn send(&self, t: T) -> Result<(), SendError<T>> ; 
    fn recv(&mut self) -> Result<R, RecvError>;

}

type CoreDuplex = dyn Duplex<MsgWithTarget, Msg> + Send;

pub trait Transport: Stoppable {
    // fn new(hid: String, duplex_for_transport: impl Duplex<Msg, MsgWithTarget>) -> Self;
    fn run(&self) -> impl Future<Output = Result<(), Box<dyn Error>>> + Send;
}

pub trait Stoppable {
    fn stop(&self);
}

pub struct MsgWithTarget{
    pub target_hid: u32,
    pub msg: Msg
}

pub struct Core {
    state: State,
    // tx: mpsc::Sender<MsgWithTarget>,
    // rx: mpsc::Receiver<Msg>,
    duplex: Box<CoreDuplex>,
    should_stop: Arc<AtomicBool>,
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
    pub fn new(duplex_for_core:impl Duplex<MsgWithTarget, Msg> + 'static + Send) -> Self {
        Core {
            state: State::default(),
            duplex: Box::new(duplex_for_core),
            should_stop: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn get_shutdown_hook(&self) -> Arc<CoreShutdownHook>{
        Arc::new(CoreShutdownHook{should_stop: self.should_stop.clone()})
    }

    pub fn run(&mut self) {
        loop {
            match self.duplex.recv() {
                Ok(msg) => {
                    self.on_message_received(msg);
                },
                Err(e) => {
                    if self.should_stop.load(Ordering::Relaxed){
                        info!("Core channel hung up, shutting down");
                        return;
                    }
                    else {
                        warn!("Core channel hung up without clean shutdown");
                        return;
                    }
                },
            }
            
        }

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

pub struct CoreShutdownHook{
    should_stop: Arc<AtomicBool>
}
impl Stoppable for CoreShutdownHook {
    fn stop(&self) {
        self.should_stop.store(true, Ordering::Relaxed);
    }
}
