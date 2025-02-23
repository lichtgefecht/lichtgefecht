use log::warn;

use crate::Core;

use super::MessageHandler;

pub struct UnimplementedMessageHandler {
    msg: String,
}
impl UnimplementedMessageHandler {
    pub fn new(msg: String) -> Box<Self> {
        Box::new(UnimplementedMessageHandler { msg })
    }
}
impl MessageHandler for UnimplementedMessageHandler {
    fn handle(&self, _core: &mut Core) {
        warn!("Handling unimplemented message: {:?}", self.msg);
    }
}

pub struct IgnoredMessageHandler;

impl IgnoredMessageHandler {
    pub fn new() -> Box<Self> {
        Box::new(IgnoredMessageHandler)
    }
}
impl MessageHandler for IgnoredMessageHandler {
    fn handle(&self, _core: &mut Core) {}
}
