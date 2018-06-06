use actix::prelude::*;
use std::net::{UdpSocket, SocketAddr, IpAddr, Ipv4Addr};
use dhcp_frames::DHCPPacket;
use config::Config;

struct IoActor {
    socket: UdpSocket,
    server_actor: Addr<Syn, _>,
    bcast_addr: Ipv4Addr,
}

impl Actor for IoActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        let socket = self.socket.try_clone().unwrap();
        ctx.spawn(move || {
            loop {
                let mut buf = vec![0u8; 1024];
                let _ = socket.recv_from(&mut buf).unwrap();
                let packet = DHCPPacket::from_vec(buf).unwrap();
                self.server_actor.do_send(packet);
            }
        });
    }
}

impl Handler<DHCPPacket> for IoActor {
    type Result = ();

    fn handle(&mut self, msg: DHCPPacket, ctx: &mut Context<Self>)  {
        self.socket.send_to(msg.into_vec().as_slice(), self.bcast_addr);
    }
}

impl IoActor {
    fn new<A>(server_actor: Addr<Syn, A>, conf: Config) -> Self {
        let sock_addr = SocketAddr::new(IpAddr::from(Ipv4Addr::from(conf.gateway)), 67);
        let mut socket = UdpSocket::bind(sock_addr).unwrap();
        socket.set_broadcast(true);
        IoActor {
            socket: socket,
            server_actor: server_actor,
            bcast_addr: Ipv4Addr::from([255,255,255,255])
        }
    }
}