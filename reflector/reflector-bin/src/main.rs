use std::error::Error;
use std::io;
use tokio::net::UdpSocket;

async fn run(socket: UdpSocket) -> Result<(), io::Error> {
    let mut buf =  vec![0; 1024];
    let mut to_send = None;

    loop {
        if let Some((size, peer)) = to_send {
            let amt = socket.send_to(&buf[..size], &peer).await?;
            println!("Echoed {amt}/{size} bytes to {peer}");
        }
        to_send = Some(socket.recv_from(&mut buf).await?);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let socket = UdpSocket::bind(&"0.0.0.0:3333").await?;
    println!("Listening on: {}", socket.local_addr()?);
    run(socket).await?;
    Ok(())
}