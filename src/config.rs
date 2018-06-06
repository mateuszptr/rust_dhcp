use serde;
use serde_json;
use std::collections::HashMap;
use std::ops::Range;


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
}

pub struct Config {
    pub pool_range: Range<u32>,
    pub pool_mask: u32,
    pub dns: Vec<u32>,
    pub gateway: u32,
    pub statics: HashMap<u32, u64>,
    pub lease_time: u32,
    pub expiration_time: u32,
}

#[test]
fn serialize_config_test() {

}