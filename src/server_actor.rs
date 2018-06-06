
use actix::prelude::*;
use std::collections::HashMap;
use config::Config;
use std::iter::Cycle;
use dhcp_frames::DHCPPacket;
use dhcp_options::*;
use byteorder::{NetworkEndian, ReadBytesExt};
use std::u32;
use std::ops::Range;
use std::time::Duration;

pub enum Status {
    Leasing,
    Reserved,
    Expiring
}

struct MapEntry {
    status: Status,
    hwaddr: u64,
    spawn_handle: Option<SpawnHandle>,
}


struct ServerActor {
    lease_map: HashMap<u32, MapEntry>,
    conf: Config,
    pool_iter: Cycle<Range<u32>>,
}

impl ServerActor {

    fn ack_options(&self, message_type: u8) -> HashMap<u8, Vec<u8>> {
        let mut options = HashMap::new();

        options.insert(DHCP_MESSAGE_TYPE, vec![message_type]);
        let netmask: u32 = self.conf.pool_mask.to_be();
        options.insert(SUBNET_MASK, netmask.to_bytes().to_vec());
        let router: u32 = self.conf.gateway.to_be();
        options.insert(ROUTER, router.to_bytes().to_vec());
        let lease_time: u32 = self.conf.lease_time.to_be();
        options.insert(IP_ADDRESS_LEASE_TIME, lease_time.to_bytes().to_vec());




        options
    }

    fn nak_options(&self) -> HashMap<u8, Vec<u8>> {
        let mut options = HashMap::new();

        options
    }

    fn ack_frame(&self, message_type: u8, packet: DHCPPacket, yiaddr: u32) -> DHCPPacket {
        let mut header = packet.header;
        header.yiaddr = yiaddr;
        header.siaddr = self.conf.gateway;
        header.op = 0x02;
        header.flags = 0x01; // BE

        let options = self.ack_options(message_type);

        DHCPPacket {header, options}
    }

    fn handle_discover(&self, packet: DHCPPacket, ctx: &mut Context<Self>) {
        let mac = packet.header.chaddr;
        let wanted_ip = packet.options.get(&REQUESTED_IP_ADDRESS);
        let new_ip: u32;

        match wanted_ip {
            None => {
                new_ip = self.pool_iter.next().unwrap();
            },
            Some(v) => {
                new_ip = v.as_slice().read_u32::<NetworkEndian>().unwrap();
            }
        }

        while let Some(me) = self.lease_map.get(&new_ip) {
            if me.status == Status::Expired || me.hwaddr == packet.header.chaddr {break;}
            new_ip = self.pool_iter.next().unwrap();
        }

        let spawn_handle = ctx.notify_later::<(Status, u32)>((Status::Expiring, new_ip), Duration::from_secs(self.conf.expiration_time as u64));
        let hwaddr = packet.header.chaddr;
        let status = Status::Expiring;
        let entry = MapEntry {
            status: status,
            spawn_handle: Some(spawn_handle),
            hwaddr: hwaddr
        };

        self.lease_map.insert(new_ip, entry);

        let frame = self.ack_frame(DHCP_OFFER, packet, new_ip);
        let frame = frame.into_vec();
    }
}

impl Actor for ServerActor {
    type Context = Context<Self>;


}



impl Handler<DHCPPacket> for ServerActor {
    type Result = ();

    fn handle(&mut self, msg: DHCPPacket, ctx: &mut Context<Self>) {

    }
}

impl Handler<(Status, u32)> for ServerActor {
    type Result = ();

    fn handle(&mut self, msg: (Status, u32), ctx: &mut Context<Self>) {
        let status = msg._0;
        let ip = msg._1;

        match status {
            Status::Expiring => {
                self.lease_map.remove(&ip);
            },
            Status::Leasing => {
                let spawn_handle = ctx.notify_later::<(Status, u32)>((Status::Expiring, ip), Duration::from_secs(self.conf.expiration_time as u64));
                let entry = self.lease_map.get_mut(&ip).unwrap();
                entry.status = Status::Expiring;
                entry.spawn_handle = Some(spawn_handle);
            }
        }
    }
}