mod system_bc_reply;
mod system_unimpl;
mod system_infra;
pub mod router;

use std::fmt::Debug;

// pub use handler_bc_reply::BroadcastReplyHandler;
pub use system_unimpl::*;
pub use system_infra::*;

use crate::core::InnerCore;

pub trait System<M>
where M :MessageTrait
{
    fn handle(&mut self, state: &mut InnerCore, msg: &M);
}

pub trait MessageTrait : Debug{}
