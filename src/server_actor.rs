use actix::prelude::*;
use byteorder::{NetworkEndian, ReadBytesExt};
use config::Config;
use dhcp_frames::DHCPPacket;
use dhcp_options::*;
use io_actor::OutputActor;
use std::collections::HashMap;
use std::iter::Cycle;
use std::ops::Range;
use std::time::Duration;
use std::u32;

#[derive(PartialEq)]
pub enum Status {
    Leasing,
    Reserved,
    Expiring,
    Declined,
}

struct MapEntry {
    status: Status,
    hwaddr: u64,
    spawn_handle: Option<SpawnHandle>,
}


pub struct ServerActor {
    lease_map: HashMap<u32, MapEntry>,
    conf: Config,
    pool_iter: Cycle<Range<u32>>,
    output_actor: Addr<Syn, OutputActor>,
}

impl ServerActor {
    fn ack_options(&self, message_type: u8) -> HashMap<u8, Vec<u8>> {
        let mut options = HashMap::new();



        options.insert(DHCP_MESSAGE_TYPE, vec![message_type]);
        let netmask: u32 = self.conf.pool_mask.to_be();
        options.insert(SUBNET_MASK, netmask.to_bytes().to_vec());
        let router: u32 = self.conf.gateway.to_be();
        options.insert(ROUTER, router.to_bytes().to_vec());
        options.insert(DHCP_SERVER_IDENTIFIER, router.to_bytes().to_vec());
        let lease_time: u32 = self.conf.lease_time.to_be();
        options.insert(IP_ADDRESS_LEASE_TIME, lease_time.to_bytes().to_vec());


        options
    }

    fn nak_options(&self) -> HashMap<u8, Vec<u8>> {
        let mut options = HashMap::new();

        options
    }

    fn ack_frame(&self, message_type: u8, mut packet: DHCPPacket, yiaddr: u32) -> DHCPPacket {
        let mut header = packet.header;
        header.yiaddr = yiaddr;
        header.siaddr = self.conf.gateway;
        header.op = 0x02;
        header.flags = 0x8000;

        let options = self.ack_options(message_type);

        DHCPPacket { header, options }
    }

    fn nak_frame(&self, mut packet: DHCPPacket) -> DHCPPacket {
        let mut header = packet.header;
        header.siaddr = self.conf.gateway;
        header.op = 0x02;
        header.flags = 0x8000;

        let options = self.nak_options();
        DHCPPacket { header, options }
    }

    fn get_new_ipaddr(&mut self, wanted_ip: Option<u32>, hwaddr: u64) -> u32 {
        let mut new_ip;

        match wanted_ip {
            Some(ip) => new_ip = ip,
            None => new_ip = self.pool_iter.next().unwrap(),
        }

        while let Some(me) = self.lease_map.get(&new_ip) {
            if me.status == Status::Expiring || me.hwaddr == hwaddr { break; }
            new_ip = self.pool_iter.next().unwrap();
        }

        new_ip
    }

    fn handle_discover(&mut self, mut packet: DHCPPacket, ctx: &mut Context<Self>) {
        println!("Handling discover");
        let wanted_ip;
        {
            let wanted_ip_ = packet.options.get(&REQUESTED_IP_ADDRESS);
            wanted_ip = match wanted_ip_ {
                Some(ip) => ip.as_slice().read_u32::<NetworkEndian>().ok(),
                None => None,
            };
        }

        let mut new_ip = self.get_new_ipaddr(wanted_ip, packet.header.chaddr);

        let spawn_handle = ctx.notify_later::<StatusMessage>(StatusMessage(Status::Expiring, new_ip), Duration::from_secs(self.conf.expiration_time as u64));
        let hwaddr = packet.header.chaddr;
        let status = Status::Expiring;
        let entry = MapEntry {
            status: status,
            spawn_handle: Some(spawn_handle),
            hwaddr: hwaddr,
        };

        self.lease_map.insert(new_ip, entry);

        let frame = self.ack_frame(DHCP_OFFER, packet, new_ip);
        println!("Sending DHCPOFFER frame to output actor");
        self.output_actor.do_send::<DHCPPacket>(frame);
    }

    fn handle_request(&mut self, mut packet: DHCPPacket, ctx: &mut Context<Self>) {
        let wanted_ip;
        {
            let ciaddr = packet.header.ciaddr;
            if ciaddr != 0 {
                wanted_ip = Some(ciaddr);
            } else {
                let wanted_ip_ = packet.options.get(&REQUESTED_IP_ADDRESS);
                wanted_ip = match wanted_ip_ {
                    Some(ip) => ip.as_slice().read_u32::<NetworkEndian>().ok(),
                    None => None,
                };
            }
        }

        let mut new_ip = self.get_new_ipaddr(wanted_ip, packet.header.chaddr);


        let spawn_handle = ctx.notify_later::<StatusMessage>(StatusMessage(Status::Leasing, new_ip), Duration::from_secs(self.conf.lease_time as u64));
        let hwaddr = packet.header.chaddr;
        let status = Status::Leasing;
        let entry = MapEntry {
            status: status,
            spawn_handle: Some(spawn_handle),
            hwaddr: hwaddr,
        };

        let prev_entry = self.lease_map.remove(&new_ip);
        match prev_entry {
            None => (),
            Some(me) => match me.spawn_handle {
                Some(sh) => {ctx.cancel_future(sh);},
                None => (),
            },
        }

        self.lease_map.insert(new_ip, entry);

        let frame = self.ack_frame(DHCP_ACK, packet, new_ip);
        self.output_actor.do_send(frame);
    }

    fn handle_inform(&self, mut packet: DHCPPacket, ctx: &mut Context<Self>) {
        let frame = self.nak_frame(packet);
        self.output_actor.do_send(frame);
    }

    fn handle_release(&mut self, packet: DHCPPacket, ctx: &mut Context<Self>) {
        let rel_ip = packet.header.ciaddr;
        let entry = self.lease_map.remove(&rel_ip);
        match entry {
            Some(MapEntry { ref status, ref spawn_handle, .. }) if status == &Status::Leasing => {
                ctx.cancel_future(spawn_handle.unwrap());
                //self.lease_map.remove(&rel_ip);
            }
            Some(_) => { self.lease_map.insert(rel_ip, entry.unwrap()); }
            None => (),
        };
    }

    fn handle_decline(&mut self, packet: DHCPPacket, ctx: &mut Context<Self>) {
        let decl_ip = packet.header.ciaddr;
        let entry = self.lease_map.remove(&decl_ip);
        match entry {
            Some(MapEntry { ref status, ref spawn_handle, ref hwaddr }) if status == &Status::Leasing => {
                ctx.cancel_future(spawn_handle.unwrap());
                let new_spawn_handle = ctx.notify_later::<StatusMessage>(StatusMessage(Status::Expiring, decl_ip), Duration::from_secs(self.conf.expiration_time as u64));
                let new_entry = MapEntry {
                    status: Status::Declined,
                    hwaddr: *hwaddr,
                    spawn_handle: Some(new_spawn_handle),
                };
                self.lease_map.insert(decl_ip, new_entry);
            }
            Some(_) => { self.lease_map.insert(decl_ip, entry.unwrap()); }
            None => ()
        };
    }

    pub fn new(config: Config, output_actor: Addr<Syn, OutputActor>) -> Self {
        let pool_iter = config.pool_range.clone().cycle();
        ServerActor {
            lease_map: HashMap::new(),
            output_actor: output_actor,
            conf: config,
            pool_iter: pool_iter,
        }
    }
}

impl Actor for ServerActor {
    type Context = Context<Self>;
}


impl Handler<DHCPPacket> for ServerActor {
    type Result = ();

    fn handle(&mut self, msg: DHCPPacket, ctx: &mut Context<Self>) {
        let dhcp_message_type;
        {
            let dhcp_message_type_ = msg.options.get(&DHCP_MESSAGE_TYPE);
            dhcp_message_type = dhcp_message_type_.unwrap().as_slice().read_u8().unwrap();
        }

        println!("Got message with type {}", dhcp_message_type);

        match dhcp_message_type {
            DHCP_DISCOVER => self.handle_discover(msg, ctx),
            DHCP_REQUEST => self.handle_request(msg, ctx),
            DHCP_INFORM => self.handle_inform(msg, ctx),
            DHCP_DECLINE => self.handle_decline(msg, ctx),
            DHCP_RELEASE => self.handle_release(msg, ctx),
            _ => ()
        }
    }
}

#[derive(Message)]
struct StatusMessage(Status, u32);

impl Handler<StatusMessage> for ServerActor {
    type Result = ();

    fn handle(&mut self, msg: StatusMessage, ctx: &mut Context<Self>) {
        let status = msg.0;
        let ip = msg.1;

        match status {
            Status::Expiring | Status::Declined => {
                self.lease_map.remove(&ip);
            },
            Status::Leasing => {
                let spawn_handle = ctx.notify_later::<StatusMessage>(StatusMessage(Status::Expiring, ip), Duration::from_secs(self.conf.expiration_time as u64));
                let entry = self.lease_map.get_mut(&ip).unwrap();
                entry.status = Status::Expiring;
                entry.spawn_handle = Some(spawn_handle);
            },
            _ => ()
        }
    }
}