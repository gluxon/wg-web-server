pub mod conf_file;

pub mod interface;
pub use interface::Interface;

pub mod peer;
pub use peer::Peer;

pub mod privatekey;
pub use privatekey::PrivateKey;

pub mod publickey;
pub use publickey::PublicKey;

pub mod presharedkey;
pub use presharedkey::PresharedKey;

pub struct Config {
    pub name: String,
    pub interface: Interface,
    pub peers: Vec<Peer>,
}

use failure;
use std::fmt;
use std::fs;
use std::io;
use std::io::Write;
use std::path::Path;

impl Config {
    pub fn new(name: String) -> Result<Self, failure::Error> {
        Ok(Self {
            name,
            interface: Interface::new()?,
            peers: vec![],
        })
    }

    pub fn init_from_path(name: String, path: &Path) -> Result<Self, failure::Error> {
        match fs::File::open(path) {
            Ok(file) => Self::read_from_file(name, file),
            Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
                Self::create_new_at_path(name, path)
            }
            Err(e) => Err(e.into()),
        }
    }

    pub fn create_new_at_path(name: String, path: &Path) -> Result<Self, failure::Error> {
        let mut file = fs::File::create(path)?;
        let config = Self::new(name)?;
        file.write_all(config.to_string().as_bytes())?;
        Ok(config)
    }

    pub fn read_from_file(name: String, file: fs::File) -> Result<Self, failure::Error> {
        let mut conf = conf_file::parse(file)?;

        let mut sections = conf.drain(..);

        let mut interface_section = sections
            .next()
            .filter(|section| section.name == "Interface")
            .ok_or(ParseInvalidFirstSection)?;

        let interface = Interface {
            private_key: interface_section
                .values
                .remove("PrivateKey")
                .ok_or(ParseMissingPrivateKeyError)?
                .parse()?,
            listen_port: interface_section
                .values
                .remove("ListenPort")
                .map(|x| x.parse())
                .transpose()?,
            address: interface_section
                .values
                .remove("Address")
                .map(|val| val.split(',').map(str::parse).collect())
                .unwrap_or_else(|| Ok(vec![]))?,
        };

        let peers = sections
            .map(|section| {
                Some(section)
                    .filter(|section| section.name == "Peer")
                    .ok_or_else(|| ParseInvalidPeerSectionsError.into())
                    .and_then(|mut section| Peer::from_hashmap(&mut section.values))
            })
            .collect::<Result<Vec<Peer>, _>>()?;

        Ok(Self {
            name,
            interface,
            peers,
        })
    }
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "[Interface]")?;

        writeln!(f, "PrivateKey = {}", &self.interface.private_key)?;

        if let Some(listen_port) = &self.interface.listen_port {
            writeln!(f, "ListenPort = {}", &listen_port)?;
        }

        if !self.interface.address.is_empty() {
            let address = &self
                .interface
                .address
                .iter()
                .map(std::string::ToString::to_string)
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
#[fail(
    display = "Invalid configuration file. Only [Peer] sections are allowed after the first [Interface] section."
)]
pub struct ParseInvalidPeerSectionsError;

#[derive(Debug, failure::Fail)]
#[fail(display = "The [Interface] section is missing a private key.")]
pub struct ParseMissingPrivateKeyError;
