use crate::net::{PresharedKey, PublicKey};
use failure;
use ipnet::IpNet;
use std::collections::HashMap;
use std::fmt;
use std::net::SocketAddr;

pub struct Peer {
    pub public_key: PublicKey,
    pub preshared_key: Option<PresharedKey>,
    pub allowed_ips: Vec<IpNet>,
    // TODO: Allow endpoint to also be a hostname
    pub endpoint: Option<SocketAddr>,
    pub persistent_keepalive: Option<u16>
}

impl Peer {
    pub fn from_hashmap(values: &mut HashMap<String, String>) -> Result<Self, failure::Error> {
        Ok(Self {
            public_key: values.remove("PublicKey")
                .ok_or(ParseMissingPublicKeyError)?
                .parse()?,
            preshared_key: values.remove("PresharedKey")
                .map(|x| x.parse())
                .transpose()?,
            allowed_ips: vec![],
            endpoint: None,
            persistent_keepalive: values.remove("PersistentKeepalive")
                .map(|x| x.parse())
                .transpose()?
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

#[derive(Debug, failure::Fail)]
#[fail(display = "[Peer] section is missing required PublicKey field.")]
pub struct ParseMissingPublicKeyError;
