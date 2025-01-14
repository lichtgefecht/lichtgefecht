use std::error::Error;
use reflector_core::Core;

mod udp;

pub use udp::UdpTransport;

pub trait Transport: Stoppable {
    fn new(core: Core, hid: String) -> Self;
    #[allow(async_fn_in_trait)]
    async fn run(&self) -> Result<(), Box<dyn Error>>;
    fn send(&self);
}

pub trait Stoppable {
    fn stop(&self);
}
