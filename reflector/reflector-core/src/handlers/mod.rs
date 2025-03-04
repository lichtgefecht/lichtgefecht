mod handler_bc_reply;
mod handler_unimpl;

// pub use handler_bc_reply::BroadcastReplyHandler;
pub use handler_unimpl::*;
// pub use handler_unimpl::UnimplementedMessageHandler;

use crate::Core;

pub trait MessageHandler
// where T :MessageTrait
{
    type Message;
    fn handle(&self, core: &Core, message: &Self::Message);
}


// pub trait MessageStateVerwalter<T>
// where T :MessageTrait
// {
//     // type Message : MessageTrait;
//     fn handle(&self, core: &Core, message: &T, state: &mut MyState);
// }

// pub trait MessageTrait<S>;
pub trait MessageTrait {
}
