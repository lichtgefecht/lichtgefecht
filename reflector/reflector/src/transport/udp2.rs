use std::io::IoSliceMut;
use std::mem::MaybeUninit;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::os::fd::{AsFd, AsRawFd};
use std::{net::SocketAddr, sync::atomic::AtomicBool};

use std::sync::atomic::Ordering::Relaxed;

use libc::in_pktinfo;
use log::{error, info};
use nix::libc::c_int;
use nix::sys::socket::{recvmsg, setsockopt, ControlMessage, ControlMessageOwned, MsgFlags, RecvMsg, SockaddrIn};
use nix::sys::socket::sockopt::Ipv4PacketInfo;
use libc::in_addr;
use socket2::{Domain, MaybeUninitSlice, MsgHdrMut, Socket, Type};
use nix::cmsg_space;
use super::{Stoppable, SyncTransport};

pub struct UdpTransport2{
    shutting_down: AtomicBool,

}

impl UdpTransport2{
    pub fn new()->Self{
        UdpTransport2{
            shutting_down: AtomicBool::new(false)
        }
    }
}

impl SyncTransport for UdpTransport2{
    fn run(&self) -> Result<(), Box<dyn std::error::Error>> {


        let socket = Socket::new(Domain::IPV4, Type::DGRAM, None)?;
        let address: SocketAddr = "0.0.0.0:3333".parse().unwrap();
        let address = address.into();
        let fd = socket.as_fd();
        setsockopt(&fd, Ipv4PacketInfo, &true).unwrap();
        socket.bind(&address)?;


        let mut cmsg = cmsg_space!(in_pktinfo);

        // 1. setsockopt IP_PKTINFO
        setsockopt(&fd, Ipv4PacketInfo, &true).unwrap();
    
        loop {
            // 2. recvmsg from socket
            let buf: &mut [u8] = &mut [0; 2048];
            let iov = &mut [IoSliceMut::new(buf)];
    
            let fd = fd.as_raw_fd();
            let res:RecvMsg<'_, '_, SockaddrIn> = recvmsg(fd, iov, Some(&mut cmsg), MsgFlags::empty()).unwrap();
            match res.cmsgs().unwrap().next().unwrap() {
                ControlMessageOwned::Ipv4PacketInfo(info) => {
    
                    
                    let addr = Ipv4Addr::from_bits(info.ipi_spec_dst.s_addr.to_be());
                    info!("got {:?}", addr);
                },
                _ => break,
            };
        }

        info!("UdpTransport2 shutting down");
        Ok(())
    }

}

impl Stoppable for UdpTransport2{

    fn stop(&self) {
        self.shutting_down.store(true, Relaxed);
    }
}
