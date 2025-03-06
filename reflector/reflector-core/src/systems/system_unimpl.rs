use super::System;
use crate::{core::InnerCore, message::CoreMessage};
use log::{debug, warn};
use reflector_api::lg::BroadcastReply;

pub struct IgnoredMessageHandler;
pub struct UnimplementedMessageHandler;

impl System<CoreMessage> for IgnoredMessageHandler {
    fn handle(&mut self, _core: &mut InnerCore, msg: &CoreMessage) {
        debug!("Ignoring message: {msg:#?}")
    }
}
impl System<CoreMessage> for UnimplementedMessageHandler {
    fn handle(&mut self, _core: &mut InnerCore, msg: &CoreMessage) {
        warn!("Message handler unimplemented: {msg:#?}")
    }
}
