pub mod router;
mod system_bc_reply;
mod system_infra;
mod system_unimpl;
mod system_hit;

use std::fmt::Debug;

// pub use handler_bc_reply::BroadcastReplyHandler;
pub use system_infra::*;
pub use system_unimpl::*;
pub use system_hit::*;

use crate::core::InnerCore;

pub trait System<M>
where
    M: MessageTrait,
{
    fn handle(&mut self, state: &mut InnerCore, msg: &M);
}

pub trait MessageTrait: Debug {}
