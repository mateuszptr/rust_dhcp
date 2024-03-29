//Stałe:


pub const SUBNET_MASK: u8 = 1;
pub const ROUTER: u8 = 3;
pub const DOMAIN_NAME_SERVER: u8 = 6;

pub const REQUESTED_IP_ADDRESS: u8 = 50;
pub const IP_ADDRESS_LEASE_TIME: u8 = 51;
pub const DHCP_MESSAGE_TYPE: u8 = 53;
pub const DHCP_SERVER_IDENTIFIER: u8= 54;


pub const DHCP_OFFER: u8 = 2;
pub const DHCP_ACK: u8 = 5;
pub const DHCP_NAK: u8 = 6;

pub const DHCP_DISCOVER: u8 = 1;
pub const DHCP_REQUEST: u8 = 3;
pub const DHCP_DECLINE: u8 = 4;
pub const DHCP_INFORM: u8 = 8;
pub const DHCP_RELEASE: u8 = 7;
