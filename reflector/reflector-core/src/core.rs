use log::{debug, info, warn};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use crate::game::state::State;
use crate::{api::infra::Stoppable, systems::System};
use crate::{
    api::transport::{CoreDuplex, Duplex},
    message::{CoreMessage, OutgoingMessage},
    systems::{self, router},
};

pub struct Core {
    should_stop: Arc<AtomicBool>,
    systems: Vec<Box<dyn System<CoreMessage>>>,
    inner_core: InnerCore,
}

pub struct InnerCore {
    pub(crate) state: State,
    pub(crate) duplex: Box<CoreDuplex>,
    pub(crate) shutdown_hook: Arc<CoreShutdownHook>,
}

impl Core {
    pub fn new(
        duplex_for_core: impl Duplex<OutgoingMessage, CoreMessage> + 'static + Send,
    ) -> Self {
        let should_stop = Arc::new(AtomicBool::new(false));
        let shutdown_hook = Arc::new(CoreShutdownHook {
            should_stop: should_stop.clone(),
        });
        Core {
            should_stop,
            inner_core: InnerCore {
                state: State::default(),
                duplex: Box::new(duplex_for_core),
                shutdown_hook,
            },
            systems: router::get_registrations(),
        }
    }

    pub fn run(&mut self) {
        while !self.should_stop.load(Ordering::Relaxed) {
            match self.inner_core.duplex.recv() {
                Ok(msg) => self.on_message_received(msg),
                Err(_) => break,
            }
        }
        self.shutdown();
    }

    pub fn on_message_received(&mut self, msg: CoreMessage) {
        let systems = router::get_systems(&mut self.systems, &msg);
        for sys in systems {
            sys.handle(&mut self.inner_core, &msg);
        }
    }

    pub fn shutdown(&mut self) {
        info!("Shutting down core");
        info!("Registered clients: {:#?}", self.inner_core.state.devices);
    }

    pub fn get_shutdown_hook(&self) -> Arc<CoreShutdownHook> {
        self.inner_core.shutdown_hook.clone()
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
