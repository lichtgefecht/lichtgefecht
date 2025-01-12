use std::error::Error;

mod udp;

pub use udp::UdpTransport;

pub trait Transport: Stoppable {
    async fn run(&self) -> Result<(), Box<dyn Error>>;
}

pub trait Stoppable {
    fn stop(&self);
}
