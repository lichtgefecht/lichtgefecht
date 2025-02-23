mod handler_bc_reply;
mod handler_unimpl;

pub use handler_bc_reply::BroadcastReplyHandler;
pub use handler_unimpl::IgnoredMessageHandler;
pub use handler_unimpl::UnimplementedMessageHandler;

use crate::Core;


pub trait MessageHandler {
    fn handle(&self, core: &mut Core);
}
