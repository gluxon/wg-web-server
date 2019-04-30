use crate::states::WgState;
use askama::Template;
use failure;
use rocket::get;
use rocket::State;
use wireguard_ctrl_rs::get::Device;

#[derive(Template)]
#[template(path = "network/index.html")]
pub struct IndexTemplate {
    device: Device,
}

#[get("/")]
pub fn index(wg: State<WgState>) -> Result<IndexTemplate, failure::Error> {
    let device = wg.get_device()?;
    Ok(IndexTemplate { device })
}

mod filters {
    use askama::Error;
    use base64;
    use humantime;
    use pretty_bytes;
    use std::net::SocketAddr;
    use std::time::{Duration, SystemTime};
    use wireguard_ctrl_rs::get::AllowedIp;

    pub fn base64_encode<T: ?Sized + AsRef<[u8]>>(input: &T) -> Result<String, Error> {
        Ok(base64::encode(input))
    }

    pub fn endpoint(endpoint: &SocketAddr) -> Result<String, Error> {
        Ok(format!("{}:{}", endpoint.ip(), endpoint.port()))
    }

    pub fn allowed_ips(ips: &Vec<AllowedIp>) -> Result<String, Error> {
        Ok(ips
            .iter()
            .map(|allowed_ip| format!("{}/{}", allowed_ip.ipaddr, allowed_ip.cidr_mask))
            .collect::<Vec<String>>()
            .join(", "))
    }

    pub fn last_handshake_time(last_handshake_time: &Duration) -> Result<String, Error> {
        let difference = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .ok()
            .and_then(|now| now.checked_sub(*last_handshake_time));
        Ok(difference
            // Filter out unnecessary precision beyond seconds
            .map(|diff| Duration::new(diff.as_secs(), 0))
            .map(|diff| humantime::format_duration(diff).to_string())
            .unwrap_or_else(|| "Unknown".to_string()))
    }

    pub fn bytes(bytes: &u64) -> Result<String, Error> {
        Ok(pretty_bytes::converter::convert(bytes.clone() as f64))
    }
}
