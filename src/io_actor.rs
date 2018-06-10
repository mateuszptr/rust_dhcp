use actix::prelude::*;
use std::net::{UdpSocket, SocketAddr, IpAddr, Ipv4Addr};
use dhcp_frames::DHCPPacket;
use server_actor::ServerActor;
use config::Config;

/// Socket i adres do wysłania
pub struct OutputActor {
    socket: UdpSocket,
    bcast_addr: SocketAddr,
}

impl Actor for OutputActor {
    type Context = Context<Self>;
}

impl Handler<DHCPPacket> for OutputActor {
    type Result = ();

    /// Wysyłamy otrzymane wiadomości na socket, na adres do broadcastu.
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