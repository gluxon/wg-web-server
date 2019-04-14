use derive_builder::Builder;
use libc::timespec;
use std::cmp;
use std::fmt;
use std::net::{IpAddr, SocketAddr};

#[derive(Builder, Debug, PartialEq)]
pub struct Device {
    pub ifindex: u32,
    pub ifname: String,
    pub private_key: [u8; 32],
    pub public_key: [u8; 32],
    pub listen_port: u16,
    pub fwmark: u32,
    pub peers: Vec<Peer>,
}

#[derive(Builder, Clone, Debug, PartialEq)]
pub struct Peer {
    pub public_key: [u8; 32],
    pub preshared_key: [u8; 32],
    pub endpoint: SocketAddr,
    pub persistent_keepalive_interval: u16,
    pub last_handshake_time: LastHandshakeTime,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub allowed_ips: Vec<AllowedIp>,
    pub protocol_version: u32,
}

#[derive(Builder, Clone, Debug, PartialEq)]
pub struct AllowedIp {
    pub family: u16,
    pub ipaddr: IpAddr,
    pub cidr_mask: u8,
}

#[derive(Clone)]
pub struct LastHandshakeTime(timespec);

// The timespec struct doesn't implement Debug. Working around this with a wrapper tuple struct.
impl fmt::Debug for LastHandshakeTime {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "timespec {{ tv_sec: {}, tv_nsec: {} }}",
            self.0.tv_sec, self.0.tv_nsec
        )
    }
}

impl cmp::PartialEq for LastHandshakeTime {
    fn eq(&self, other: &LastHandshakeTime) -> bool {
        (self.0.tv_sec == other.0.tv_sec) && (self.0.tv_nsec == other.0.tv_nsec)
    }
}

impl From<timespec> for LastHandshakeTime {
    fn from(ts: timespec) -> Self {
        Self(ts)
    }
}
