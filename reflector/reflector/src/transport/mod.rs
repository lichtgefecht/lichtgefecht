use std::error::Error;

mod udp;

pub use udp::UdpTransport;

pub trait Transport {
    async fn run(&self) -> Result<(), Box<dyn Error>>;
    fn stop(&self);
}
