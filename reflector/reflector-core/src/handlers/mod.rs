mod handler_bc_reply;
mod handler_unimpl;

// pub use handler_bc_reply::BroadcastReplyHandler;
pub use handler_unimpl::*;
// pub use handler_unimpl::UnimplementedMessageHandler;

use crate::Core;

pub trait MessageHandler // where T : MsgMarker + Sized + 'static
{
    type Message : MsgMarker;
    fn handle(&self, core: &Core, message: &Self::Message);
}

pub trait MsgMarker {}
