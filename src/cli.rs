use clap::{clap_app, crate_name, crate_version};
use failure::{format_err, Error};
use std::path::PathBuf;

pub struct Args {
    pub bind_ip: String,
    pub db_path: String,
    pub foreground: bool,
    pub interface: String,
    pub interface_config: PathBuf,
    pub port: u16,
}

impl Args {
    pub fn get_from_clap() -> Result<Self, Error> {
        let matches = clap_app!(myapp =>
            (version: (crate_version!()))
            (@arg BIND_IP: -b --bind default_value("localhost"))
            (@arg DB_PATH: -d --("database-path") +takes_value)
            (@arg FOREGROUND: -f --foreground)
            (@arg INTERFACE_CONFIG: -c --("interface-config") +takes_value)
            (@arg PORT: -p --port default_value("8000"))
            // Not sure if wg0 is a good default, or if we should require this.
            (@arg INTERFACE: default_value("wg0"))
        )
        .get_matches();

        let interface = matches.value_of("INTERFACE").unwrap().to_string();

        let root_dir = format!("/var/lib/{}", crate_name!());
        let default_interface_config = format!("{}/{}.conf", root_dir, interface);

        Ok(Self {
            bind_ip: matches.value_of("BIND_IP").unwrap().to_string(),
            db_path: matches.value_of("DB_PATH").map_or_else(
                || format!("{}/{}.sqlite3", root_dir, interface),
                |x| x.to_string(),
            ),
            foreground: matches.is_present("FOREGROUND"),
            interface,
            interface_config: PathBuf::from(
                matches
                    .value_of("INTERFACE_CONFIG")
                    .unwrap_or(&default_interface_config),
            ),
            port: matches
                .value_of("PORT")
                .unwrap()
                .parse::<u16>()
                .map_err(|_| {
                    format_err!(
                        "port must be an integer in the range [0, {}]",
                        2u32.pow(16) - 1
                    )
                })?,
        })
    }
}
