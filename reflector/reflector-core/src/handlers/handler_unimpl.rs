use std::marker::PhantomData;

use crate::Core;

use super::{MessageHandler, MessageTrait};



pub struct IgnoredMessageHandler;

impl IgnoredMessageHandler {
    pub fn new() -> Self {
        IgnoredMessageHandler
    }
}

pub struct MsgFoo;
pub struct MsgBar;

pub struct SomeState;


//????
// impl MessageTrait for MsgFoo{}

// impl MessageTrait for MsgBar {}
// impl MessageTrait for Box<MsgFoo> {}
// impl MessageTrait for Box<MsgBar> {}
// impl MessageTrait for Box<dyn MessageTrait> {}

impl MessageHandler for IgnoredMessageHandler {
    type Message = MsgFoo;
    fn handle(&self, _core: &Core, message: &Self::Message) {}    
}

// impl MessageHandler<MsgBar> for IgnoredMessageHandler {
//     fn handle(&self, _core: &Core, message: &MsgBar) {}
// }

// impl MessageHandler for IgnoredMessageHandler {
//     type Message = Box<MsgFoo>;
//     fn handle(&self, _core: &Core, message: &Self::Message) {}
// }

// impl MessageHandler for IgnoredMessageHandler {
//     type Message = Box<MsgBar>;
//     fn handle(_core: &mut Core, message: &Self::Message) {}
// }
