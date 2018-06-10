#![feature(int_to_from_bytes)]

extern crate byteorder;
extern crate bytes;

extern crate serde;
extern crate serde_json;
#[macro_use] extern crate serde_derive;

extern crate actix;
#[macro_use] extern crate actix_derive;

extern crate libc;

mod dhcp_frames;
mod dhcp_options;
mod config;

mod server_actor;
mod io_actor;

use std::thread;
use std::fs::File;
use std::io::prelude::*;
use config::*;
use std::net::{UdpSocket, SocketAddr, IpAddr, Ipv4Addr};
use actix::prelude::*;
use io_actor::OutputActor;
use server_actor::ServerActor;
use dhcp_frames::DHCPPacket;
use std::os::unix::io::AsRawFd;
use std::ffi::CString;
use libc::c_void;

unsafe fn set_socket_device(socket: &UdpSocket, iface: &str) {
    let fd = socket.as_raw_fd();
    let lvl = libc::SOL_SOCKET;
    let name = libc::SO_BINDTODEVICE;

    let val = CString::new(iface).unwrap();
    let pointer = val.as_ptr() as *const c_void;
    let len = val.as_bytes_with_nul().len();

    libc::setsockopt(
        fd,
        lvl,
        name,
        pointer,
        len as libc::socklen_t
    );

}

fn main() {
    let system = actix::System::new("dhcp");

    let mut config_file = File::open("Config.json").expect("Couldn't open config file");
    let mut config_content = String::new();
    config_file.read_to_string(&mut config_content).expect("Couldn't read config file");
    let config = get_config(config_content);

    let socket = UdpSocket::bind(SocketAddr::new(IpAddr::from(Ipv4Addr::from([0,0,0,0])), 67)).expect("Couldn't bind a socket");
    unsafe { set_socket_device(&socket, config.interface.as_str()); }
    socket.set_broadcast(true).expect("Couldn't set socket to bcast");
    let input_socket = socket.try_clone().expect("Couldn't clone the socket");

    let output_actor: Addr<Syn, _> = OutputActor::new(config.clone(), socket).start();
    let server_actor: Addr<Syn, _> = ServerActor::new(config, output_actor.clone()).start();

    let input_thread_handle = thread::spawn(move || {
        loop {
            println!("Creating buffer");
            let mut buf = vec![0u8; 1024];
            let (_, addr) = input_socket.recv_from(&mut buf).unwrap();
            println!("Received frame from {}", addr);
            let packet = DHCPPacket::from_vec(buf).unwrap();
            server_actor.do_send(packet);
        }
    });

    system.run();

}
