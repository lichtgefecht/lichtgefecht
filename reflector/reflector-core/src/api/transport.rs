use reflector_api::lg::broadcast_reply::ClientAddr;
use std::sync::mpsc::{RecvError, SendError};
use std::{error::Error, future::Future};

use crate::message::OutgoingMessage;
use crate::CoreMessage;

use super::infra::Stoppable;

pub type CoreDuplex = dyn Duplex<OutgoingMessage, CoreMessage> + Send;

pub trait Duplex<T, R> {
    fn send(&self, t: T) -> Result<(), SendError<T>>;
    fn recv(&mut self) -> Result<R, RecvError>;
}

pub trait Transport: Stoppable {
    fn run(&self) -> impl Future<Output = Result<(), Box<dyn Error>>> + Send;
}
