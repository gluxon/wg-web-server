use crate::net::{Peer, PrivateKey};
use crate::net::conf::parse;
use failure;
use ipnet::IpNet;
use std::fmt;
use std::fs;
use std::io;
use std::io::Write;
use std::path::Path;

pub struct Interface {
    // wg fields
    pub private_key: PrivateKey,
    pub listen_port: Option<u16>,

    // wg-quick fields
    pub address: Vec<IpNet>,
    // TODO: There are more wg-quick fields that we should support and add here.
    // https://github.com/WireGuard/WireGuard/blob/516f05862edd0bb135dcb9c93acaa67ff5b676ed/src/tools/man/wg-quick.8#L68

    // wg-web-server specific (serialized as their own INI-style sections)
    pub peers: Vec<Peer>,
}

impl Interface {
    pub fn new() -> Result<Self, failure::Error> {
        Ok(Self {
            private_key: PrivateKey::new()?,
            listen_port: None,
            address: vec![],
            peers: vec![]
        })
    }

    pub fn init_from_path(path: &Path) -> Result<Self, failure::Error> {
        match fs::File::open(path) {
            Ok(file) => Self::read_from_file(file),
            Err(ref e) if e.kind() == io::ErrorKind::NotFound =>
                Self::create_new_at_path(path),
            Err(e) => return Err(e.into())
        }
    }

    pub fn create_new_at_path(path: &Path) -> Result<Self, failure::Error> {
        let mut file = fs::File::create(path)?;
        let interface = Interface::new()?;
        file.write_all(interface.to_string().as_bytes())?;
        Ok(interface)
    }

    pub fn read_from_file(file: fs::File) -> Result<Self, failure::Error> {
        let mut conf = parse(file)?;

        let mut sections = conf.sections.drain(..);

        let mut interface_section = sections.next()
            .filter(|section| section.name == "Interface")
            .ok_or(ParseInvalidFirstSection)?;

        let interface = Interface {
            private_key: interface_section.values.remove("PrivateKey")
                .ok_or(ParseMissingPrivateKeyError)?
                .parse()?,
            listen_port: interface_section.values.remove("ListenPort")
                .map(|x| x.parse())
                .transpose()?,
            address: interface_section.values.remove("Address")
                .map(|val| val
                    .split(",")
                    .map(|x| x.parse())
                    .collect())
                .unwrap_or_else(|| Ok(vec![]))?,
            peers: sections
                .map(|section| Some(section)
                    .filter(|section| section.name == "Peer")
                    .ok_or(ParseInvalidPeerSectionsError.into())
                    .and_then(|mut section| Peer::from_hashmap(&mut section.values)))
                .collect::<Result<Vec<Peer>, _>>()?
        };

        Ok(interface)
    }
}

impl fmt::Display for Interface {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "[Interface]")?;

        writeln!(f, "PrivateKey = {}", &self.private_key)?;

        if let Some(listen_port) = &self.listen_port {
            writeln!(f, "ListenPort = {}", &listen_port)?;
        }

        if !self.address.is_empty() {
            let address = &self.address
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>()
                .join(",");
            writeln!(f, "Address = {}", &address)?;
        };

        // TODO: Serialize the remaining fields.

        for peer in &self.peers {
            writeln!(f)?;
            write!(f, "{}", &peer)?;
        }

        Ok(())
    }
}

#[derive(Debug, failure::Fail)]
#[fail(display = "Configuration files must start with an [Interface] section.")]
pub struct ParseInvalidFirstSection;

#[derive(Debug, failure::Fail)]
#[fail(display = "Invalid configuration file. Only [Peer] sections are allowed after the first [Interface] section.")]
pub struct ParseInvalidPeerSectionsError;

#[derive(Debug, failure::Fail)]
#[fail(display = "The [Interface] section is missing a private key.")]
pub struct ParseMissingPrivateKeyError;
