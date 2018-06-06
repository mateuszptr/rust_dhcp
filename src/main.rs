#![feature(int_to_from_bytes)]

extern crate byteorder;
extern crate bytes;

extern crate serde;
extern crate serde_json;
#[macro_use] extern crate serde_derive;

extern crate actix;
#[macro_use] extern crate actix_derive;

mod dhcp_frames;
mod dhcp_options;
mod config;

mod server_actor;
mod io_actor;


fn main() {
    println!("Hello, world!");
}
