use log::warn;

use crate::Core;

use super::{MessageHandler, MsgMarker};

// pub struct UnimplementedMessageHandler {
//     msg: String,
// }
// impl UnimplementedMessageHandler {
//     pub fn new(msg: String) -> Box<Self> {
//         Box::new(UnimplementedMessageHandler { msg })
//     }
// }
// impl MessageHandler for UnimplementedMessageHandler {
//     fn handle(&self, _core: &mut Core) {
//         warn!("Handling unimplemented message: {:?}", self.msg);
//     }
// }

pub struct IgnoredMessageHandler;

impl IgnoredMessageHandler {
    pub fn new() -> Self {
        IgnoredMessageHandler
    }
}
// impl MessageHandler for IgnoredMessageHandler {
//     fn handle(&self, _core: &mut Core) {}
// }
pub struct Foo;
pub struct Bar;
impl MsgMarker for Foo {}
impl MsgMarker for Bar {}
impl MsgMarker for Box<Foo> {}
impl MsgMarker for Box<Bar> {}
impl MsgMarker for Box<dyn MsgMarker> {}

impl MessageHandler for IgnoredMessageHandler {
    type Message = Box<Foo>;
    fn handle(&self, _core: &Core, message: &Self::Message) {}
}
// impl MessageHandler for IgnoredMessageHandler {
//     type Message = Bar;
//     fn handle(_core: &mut Core, message: Box<Bar>) {}
// }
