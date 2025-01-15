use std::{error::Error, future::Future};
use reflector_core::Core;

mod udp;

pub use udp::UdpTransport;

pub trait Transport: Stoppable {
    fn new(core: Core, hid: String) -> Self;
    fn run(&self) -> impl Future<Output = Result<(), Box<dyn Error>>> + Send;
    fn send(&self);
}

pub trait Stoppable {
    fn stop(&self);
}
