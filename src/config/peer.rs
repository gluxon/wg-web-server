use super::{PresharedKey, PublicKey};
use core::str::FromStr;
use failure;
use std::collections::HashMap;
use std::fmt;
use std::net::IpAddr;
use std::net::SocketAddr;

pub struct AllowedIp {
    pub addr: IpAddr,
    pub cidr: Option<u8>,
}

impl FromStr for AllowedIp {
    type Err = failure::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tokens: Vec<&str> = s.trim().split('/').collect();
        Ok(Self {
            addr: tokens[0].parse()?,
            cidr: tokens.get(1).map(|x| x.parse()).transpose()?,
        })
    }
}

pub struct Peer {
    pub public_key: PublicKey,
    pub preshared_key: Option<PresharedKey>,
    pub allowed_ips: Vec<AllowedIp>,
    // TODO: Allow endpoint to also be a hostname
    pub endpoint: Option<SocketAddr>,
    pub persistent_keepalive: Option<u16>,
}

impl Peer {
    pub fn from_hashmap(values: &mut HashMap<String, String>) -> Result<Self, failure::Error> {
        Ok(Self {
            public_key: values
                .remove("PublicKey")
                .ok_or(ParseMissingPublicKeyError)?
                .parse()?,
            preshared_key: values
                .remove("PresharedKey")
                .map(|x| x.parse())
                .transpose()?,
            allowed_ips: values
                .remove("AllowedIPs")
                .unwrap_or_else(|| "".to_string())
                .split(',')
                .map(str::trim)
                .map(AllowedIp::from_str)
                .collect::<Result<Vec<_>, _>>()?,
            endpoint: None,
            persistent_keepalive: values
                .remove("PersistentKeepalive")
                .map(|x| x.parse())
                .transpose()?,
        })
    }
}

impl fmt::Display for Peer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "[Peer]")?;

        writeln!(f, "PublicKey = {}", &self.public_key)?;

        if let Some(preshared_key) = &self.preshared_key {
            writeln!(f, "PresharedKey = {}", &preshared_key)?;
        }

        Ok(())
    }
}

impl<'a> From<&'a Peer> for wireguard_uapi::set::Peer<'a> {
    fn from(config_peer: &'a Peer) -> Self {
        let mut peer = Self::from_public_key(config_peer.public_key.as_bytes());

        if let Some(preshared_key) = &config_peer.preshared_key {
            peer = peer.preshared_key(preshared_key.as_bytes());
        }

        if let Some(endpoint) = &config_peer.endpoint {
            peer = peer.endpoint(endpoint)
        }

        if let Some(persistent_keepalive) = config_peer.persistent_keepalive {
            peer = peer.persistent_keepalive_interval(persistent_keepalive)
        }

        let allowed_ips = (&config_peer.allowed_ips)
            .iter()
            .map(|ip| wireguard_uapi::set::AllowedIp {
                ipaddr: &ip.addr,
                cidr_mask: ip.cidr,
            })
            .collect();
        peer = peer.allowed_ips(allowed_ips);

        peer
    }
}

#[derive(Debug, failure::Fail)]
#[fail(display = "[Peer] section is missing required PublicKey field.")]
pub struct ParseMissingPublicKeyError;
