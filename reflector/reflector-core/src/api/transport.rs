use reflector_api::lg::{broadcast_reply::ClientAddr, Msg};
use std::sync::mpsc::{RecvError, SendError};
use std::{
    collections::HashMap,
    error::Error,
    fmt::Debug,
    future::Future,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use log::{info, warn};

use super::infra::Stoppable;

pub type CoreDuplex = dyn Duplex<MsgWithTarget, Msg> + Send;



pub struct MsgWithTarget {
    pub target_hid: String,
    pub msg: Msg,
}

pub trait Duplex<T, R> {
    fn send(&self, t: T) -> Result<(), SendError<T>>;
    fn recv(&mut self) -> Result<R, RecvError>;
}


pub trait Transport: Stoppable {
    fn run(&self) -> impl Future<Output = Result<(), Box<dyn Error>>> + Send;
}

pub trait TransportHandle {
    fn add_address_entry(&self, hid: String, addr: ClientAddr);
}