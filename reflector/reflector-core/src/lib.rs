use reflector_api::lg::{broadcast_reply::ClientAddr, Msg};
use std::sync::mpsc::{RecvError, SendError};
use std::{
    collections::HashMap,
    error::Error,
    fmt::Debug,
    future::Future,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use log::{info, warn};

mod handlers;
mod mapper;

pub struct Game {}

pub struct Team {}

#[derive(Debug)]
#[allow(dead_code)]
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

pub trait Duplex<T, R> {
    fn send(&self, t: T) -> Result<(), SendError<T>>;
    fn recv(&mut self) -> Result<R, RecvError>;
}

type CoreDuplex = dyn Duplex<MsgWithTarget, Msg> + Send;

pub trait Transport: Stoppable {
    fn run(&self) -> impl Future<Output = Result<(), Box<dyn Error>>> + Send;
}

pub trait TransportHandle {
    fn add_address_entry(&self, hid: String, addr: ClientAddr);
}

pub trait Stoppable {
    fn stop(&self);
}

pub struct MsgWithTarget {
    pub target_hid: String,
    pub msg: Msg,
}

pub struct Core {
    state: State,
    duplex: Box<CoreDuplex>,
    should_stop: Arc<AtomicBool>,
    handle: Arc<dyn TransportHandle + Send + Sync>,
}

pub trait MessageHandler {
    fn handle(&self, core: &mut Core);
}

impl Core {
    pub fn new(
        duplex_for_core: impl Duplex<MsgWithTarget, Msg> + 'static + Send,
        handle: Arc<dyn TransportHandle + Send + Sync>,
    ) -> Self {
        Core {
            state: State::default(),
            duplex: Box::new(duplex_for_core),
            should_stop: Arc::new(AtomicBool::new(false)),
            handle,
        }
    }

    pub fn run(&mut self) {
        loop {
            match self.duplex.recv() {
                Ok(msg) => self.on_message_received(msg),
                Err(_) if self.should_stop.load(Ordering::Relaxed) => {
                    info!("Core channel hung up, shutting down");
                    break;
                }
                Err(_) => {
                    warn!("Core channel hung up without clean shutdown");
                    break;
                }
            }
        }
        self.shutdown();
    }

    pub fn on_message_received(&mut self, msg: Msg) {
        let handler = mapper::to_message_handler(msg);
        handler.handle(self);
    }
    pub fn shutdown(&mut self) {
        info!("Shutting down core");
        info!("Registered clients: {:#?}", self.state.devices);
    }
    pub fn get_shutdown_hook(&self) -> Arc<CoreShutdownHook> {
        Arc::new(CoreShutdownHook {
            should_stop: self.should_stop.clone(),
        })
    }
}

pub struct CoreShutdownHook {
    should_stop: Arc<AtomicBool>,
}
impl Stoppable for CoreShutdownHook {
    fn stop(&self) {
        self.should_stop.store(true, Ordering::Relaxed);
    }
}
