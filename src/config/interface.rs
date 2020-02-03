use super::PrivateKey;
use failure;
use ipnet::IpNet;

pub struct Interface {
    // wg fields
    pub private_key: PrivateKey,
    pub listen_port: Option<u16>,

    // wg-quick fields
    pub address: Vec<IpNet>,
    // TODO: There are more wg-quick fields that we should support and add here.
    // https://github.com/WireGuard/WireGuard/blob/516f05862edd0bb135dcb9c93acaa67ff5b676ed/src/tools/man/wg-quick.8#L68
}

impl Interface {
    pub fn new() -> Result<Self, failure::Error> {
        Ok(Self {
            private_key: PrivateKey::new()?,
            listen_port: None,
            address: vec![],
        })
    }
}
