use super::System;
use crate::{api::infra::Stoppable, core::InnerCore, message::CoreMessage};
use log::{debug, info, warn};

pub struct InfraSystem;

impl System<CoreMessage> for InfraSystem {
    fn handle(&mut self, core: &mut InnerCore, msg: &CoreMessage) {
        match msg {
            CoreMessage::Shutdown => {
                core.shutdown_hook.stop();
                info!("Shutting down via infra system");
            }
            _ => (),
        }
    }
}
