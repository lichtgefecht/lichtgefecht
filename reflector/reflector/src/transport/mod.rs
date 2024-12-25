use std::error::Error;

mod udp;
mod udp2;

pub use udp::UdpTransport;
pub use udp2::UdpTransport2;

pub trait Transport : Stoppable {
    async fn run(&self) -> Result<(), Box<dyn Error>>;
}

pub trait SyncTransport : Stoppable{
    fn run(&self) -> Result<(), Box<dyn Error>>;
}

pub trait Stoppable{
    fn stop(&self);
}
