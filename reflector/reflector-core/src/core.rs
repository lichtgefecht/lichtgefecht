use reflector_api::lg::Msg;
use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    };
use log::{info, warn};

use crate::api::infra::Stoppable;
use crate::api::transport::{CoreDuplex, Duplex, MsgWithTarget, TransportHandle};
use crate::game::state::State;
use crate::mapper;

pub struct Core {
    pub (crate) state: State,
    duplex: Box<CoreDuplex>,
    should_stop: Arc<AtomicBool>,
    pub (crate) handle: Arc<dyn TransportHandle + Send + Sync>,
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