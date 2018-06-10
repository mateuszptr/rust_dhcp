use actix::prelude::*;
use std::net::{UdpSocket, SocketAddr, IpAddr, Ipv4Addr};
use dhcp_frames::DHCPPacket;
use server_actor::ServerActor;
use config::Config;

pub struct OutputActor {
    socket: UdpSocket,
    bcast_addr: SocketAddr,
}

impl Actor for OutputActor {
    type Context = Context<Self>;

//    fn started(&mut self, ctx: &mut Context<Self>) {
////        let socket = self.socket.try_clone().unwrap();
////        ctx.spawn(move || {
////            loop {
////                let mut buf = vec![0u8; 1024];
////                let _ = socket.recv_from(&mut buf).unwrap();
////                let packet = DHCPPacket::from_vec(buf).unwrap();
////                self.server_actor.do_send(packet);
////            }
////        });
//    }
}

impl Handler<DHCPPacket> for OutputActor {
    type Result = ();

    fn handle(&mut self, msg: DHCPPacket, ctx: &mut Context<Self>)  {
        println!("Sending frame to {}", self.bcast_addr);
        self.socket.send_to(msg.into_vec().as_slice(), self.bcast_addr);
    }
}

impl OutputActor {
    pub fn new(conf: Config, socket: UdpSocket) -> Self {
        let sock_addr = SocketAddr::new(IpAddr::from(Ipv4Addr::from(conf.gateway)), 67);
        OutputActor {
            socket: socket,
            bcast_addr: SocketAddr::new(IpAddr::from(Ipv4Addr::from([255,255,255,255])), 68)
        }
    }
}