use log::{info, warn};
use reflector_api::lg::Msg;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use crate::api::transport::{CoreDuplex, Duplex, MsgWithTarget, TransportHandle};
use crate::game::state::State;
use crate::mapper;
use crate::{
    api::infra::Stoppable,
    handlers::{Foo, IgnoredMessageHandler, MessageHandler, MsgMarker},
};

pub struct Core {
    pub(crate) state: State,
    duplex: Box<CoreDuplex>,
    should_stop: Arc<AtomicBool>,
    pub(crate) handle: Arc<dyn TransportHandle + Send + Sync>,
    handlers: Vec<Box<dyn MessageHandler<Message = Box<dyn MsgMarker>>+ Send + Sync>>,
}

impl Core {
    pub fn new(
        duplex_for_core: impl Duplex<MsgWithTarget, Msg> + 'static + Send,
        handle: Arc<dyn TransportHandle + Send + Sync>,
    ) -> Self {
        let f = Box::new(IgnoredMessageHandler{}) as Box<dyn MessageHandler<Message = Box<Foo>>>;
        let f = f as Box<dyn MessageHandler<Message = Box<dyn MsgMarker>>>;

        Core {
            state: State::default(),
            duplex: Box::new(duplex_for_core),
            should_stop: Arc::new(AtomicBool::new(false)),
            handle,
            handlers: vec![
                // Box::new(IgnoredMessageHandler {})
                ]
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

        let bmsg = Box::new(Foo);
        let handler = self.handlers.get(0).unwrap();
        // handler.handle(self, &Foo);


        // let handler = self.to_message_handler(0);

        // let bmsg = Box::new(Foo);
        // // let handler = IgnoredMessageHandler::handle;
        // // handler(self, &Foo);
        // let handler = IgnoredMessageHandler {};
        // handler.handle(self, &Foo);
        // handler(self, &(Box::new(Foo) as Box<dyn MsgMarker+ 'static>));
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
    // pub fn to_message_handler(&self, id: usize) {
    // // pub fn to_message_handler(&self, id: usize) -> &Box<dyn MessageHandler<Message = Foo>+ Send + Sync>{
    // }
}

pub struct CoreShutdownHook {
    should_stop: Arc<AtomicBool>,
}
impl Stoppable for CoreShutdownHook {
    fn stop(&self) {
        self.should_stop.store(true, Ordering::Relaxed);
    }
}
