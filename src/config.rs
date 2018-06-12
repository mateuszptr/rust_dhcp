use std::collections::HashMap;
use std::ops::Range;
use std::net::Ipv4Addr;
use serde_json;
use byteorder::{ReadBytesExt, NetworkEndian};
use std::io::Cursor;
use hwaddr::HwAddr;

// Surowa konfiguracja zebrana z JSONA
#[derive(Serialize, Deserialize)]
struct RawConfig {
    pool_start: String,
    pool_end: String,
    pool_mask: String,
    dns: Vec<String>,
    gateway: String,
    statics: HashMap<String, String>,
    lease_time: u32,
    expiration_time: u32,
    iface: String,
}

// Konfiguracja, która trafi do aktorów. Odpowiednio: pula adresów, maska, dnsy, brama i adres serwera DHCP, statyczne przydziały, czas dzierżawy, czas na który trzymamy adres po wygaśnięciu, interfejs gniazda
#[derive(Clone)]
pub struct Config {
    pub pool_range: Range<u32>,
    pub pool_mask: u32,
    pub dns: Vec<u32>,
    pub gateway: u32,
    pub statics: HashMap<u32, u64>,
    pub lease_time: u32,
    pub expiration_time: u32,
    pub interface: String,
}

// Ip w formacie 192.168.0.1 na liczbę całkowitą
fn get_ip(text: &String) -> u32 {
    let ip: Ipv4Addr = text.parse().unwrap();
    let mut octets = Cursor::new(ip.octets());
    octets.read_u32::<NetworkEndian>().unwrap()
}

pub fn get_config(text: String) -> Config {
    let raw_config: RawConfig = serde_json::from_str(&text).unwrap();
    let pool_range = get_ip(&raw_config.pool_start) .. get_ip(&raw_config.pool_end)+1;
    let dns: Vec<u32> = raw_config.dns.iter().map(|text| get_ip(text)).collect();
    let mut statics: HashMap<u32, u64> = HashMap::new();

    for (ip, mac) in raw_config.statics {
        let ip = get_ip(&ip);
        let mac = mac.parse::<HwAddr>().unwrap().octets();
        let mac = Cursor::new(mac).read_uint::<NetworkEndian>(6).unwrap();

        statics.insert(ip, mac);
    }

    Config {
        pool_range: pool_range,
        pool_mask: get_ip(&raw_config.pool_mask),
        dns: dns,
        gateway: get_ip(&raw_config.gateway),
        statics: statics,
        lease_time: raw_config.lease_time,
        expiration_time: raw_config.expiration_time,
        interface: raw_config.iface,
    }
}

/// wygenerowanie przykładowej konfiguracji
#[test]
fn serialize_config_test() {
    let mut statics = HashMap::new();
    statics.insert(String::from("192.168.0.3"), String::from("FF:FF:FF:FF:FF:FF"));

    let config = RawConfig {
        pool_start: String::from("192.168.0.2"),
        pool_end: String::from("192.168.0.100"),
        pool_mask: String::from("255.255.255.0"),
        dns: vec![String::from("4.4.4.4"), String::from("8.8.8.8")],
        gateway: String::from("192.168.0.1"),
        statics: statics,
        lease_time: 300,
        expiration_time: 300,
        iface: String::from("eth0"),
    };

    println!("{}", serde_json::to_string(&config).unwrap());
}