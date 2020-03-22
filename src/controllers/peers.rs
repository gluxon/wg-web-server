use crate::config;
use crate::config::peer::AllowedIps;
use crate::config::{PresharedKey, PublicKey};
use crate::lang;
use crate::states::WgState;
use crate::utils::FormInputResult;
use crate::utils::FormOption;
use askama::Template;
use rocket::http::RawStr;
use rocket::http::Status;
use rocket::request::Form;
use rocket::response::status;
use rocket::{get, State};
use rocket::{post, FromForm};
use std::borrow::Cow;
use std::default::Default;
use std::net::SocketAddr;

#[derive(Default, Template)]
#[template(path = "peers/add.html")]
pub struct AddPeerTemplate<'a> {
    status: Option<Cow<'a, str>>,
    public_key_err: Option<String>,
    preshared_key_err: Option<String>,
    allowed_ips_err: Option<String>,
    endpoint_err: Option<String>,
    persistent_keepalive_err: Option<String>,
}

#[get("/add")]
pub fn add() -> AddPeerTemplate<'static> {
    AddPeerTemplate::default()
}

#[derive(FromForm)]
pub struct AddPeer<'v> {
    public_key: FormInputResult<'v, PublicKey>,
    preshared_key: FormOption<FormInputResult<'v, PresharedKey>>,
    allowed_ips: FormOption<FormInputResult<'v, AllowedIps>>,
    // TODO: Allow endpoint to also be a hostname
    endpoint: FormOption<Result<SocketAddr, &'v RawStr>>,
    persistent_keepalive: FormOption<Result<u16, &'v RawStr>>,
}

#[post("/add", data = "<form>")]
pub fn post_add(
    wg: State<WgState>,
    form: Form<AddPeer>,
) -> status::Custom<AddPeerTemplate<'static>> {
    // TODO:
    //   - Record this new peer in the database.
    //   - Calculate the next available IP and give it to this peer.

    let add_peer = form.into_inner();

    let public_key = match add_peer.public_key {
        Ok(public_key) => public_key,
        Err(public_key_err) => {
            let template = AddPeerTemplate {
                public_key_err: Some(format!("{}", public_key_err.error)),
                ..Default::default()
            };
            return status::Custom(Status::BadRequest, template);
        }
    };

    let preshared_key = match add_peer.preshared_key.into() {
        Some(Ok(preshared_key)) => Some(preshared_key),
        Some(Err(preshared_key_err)) => {
            let template = AddPeerTemplate {
                preshared_key_err: Some(format!("{}", preshared_key_err.error)),
                ..Default::default()
            };
            return status::Custom(Status::BadRequest, template);
        }
        None => None,
    };

    let allowed_ips = match add_peer.allowed_ips.into() {
        Some(Ok(allowed_ips)) => allowed_ips,
        Some(Err(allowed_ips_err)) => {
            let template = AddPeerTemplate {
                allowed_ips_err: Some(format!("{}", allowed_ips_err.error)),
                ..Default::default()
            };
            return status::Custom(Status::BadRequest, template);
        }
        None => AllowedIps::new(),
    };

    let endpoint = match add_peer.endpoint.into() {
        Some(Ok(endpoint)) => Some(endpoint),
        Some(Err(_)) => {
            let template = AddPeerTemplate {
                endpoint_err: Some("huh".to_string()),
                ..Default::default()
            };
            return status::Custom(Status::BadRequest, template);
        }
        None => None,
    };

    let persistent_keepalive = match add_peer.persistent_keepalive.into() {
        Some(Ok(persistent_keepalive)) => Some(persistent_keepalive),
        Some(Err(_)) => {
            let template = AddPeerTemplate {
                persistent_keepalive_err: Some("huh".to_string()),
                ..Default::default()
            };
            return status::Custom(Status::BadRequest, template);
        }
        None => None,
    };

    let config_peer = config::Peer {
        public_key: public_key.clone(),
        preshared_key,
        allowed_ips,
        endpoint,
        persistent_keepalive,
    };

    let add_peer_result = wg.add_peer(config_peer);
    let template = AddPeerTemplate {
        status: Some(match add_peer_result {
            Ok(_) => format!("{} {}", lang::ADD_PEER_SUCCESS, public_key).into(),
            Err(_) => lang::ADD_PEER_ERROR.into(),
        }),
        ..Default::default()
    };
    status::Custom(Status::Ok, template)
}

#[cfg(test)]
mod tests {
    use crate::config::peer::AllowedIps;
    use crate::config::{PresharedKey, PublicKey};
    use crate::db::make_rocket_database_config;
    use crate::launchpad;
    use crate::states::WgState;
    use failure;
    use failure::format_err;
    use rocket::config::{Config, Environment};
    use rocket::http::uri::Uri;
    use rocket::http::{ContentType, Status};
    use rocket::local::Client;
    use rocket::Rocket;
    use std::net::SocketAddr;
    use std::path::PathBuf;
    use std::str::FromStr;
    use wireguard_uapi::{get, DeviceInterface, WgSocket};

    fn get_test_rocket(db_path_buf: PathBuf) -> Result<Rocket, failure::Error> {
        let interface_config = crate::config::Config {
            name: "wgtest".to_owned(),
            interface: crate::config::Interface::new()?,
            peers: vec![],
        };
        let wgstate = WgState::init(interface_config)?;
        wgstate.apply_config()?;

        let db_path = db_path_buf
            .into_os_string()
            .into_string()
            .map_err(|os_string| format_err!("Failed to convert OsString: {:?}", os_string))?;
        let config = Config::build(Environment::Development)
            .extra("databases", make_rocket_database_config(&db_path))
            .finalize()?;

        Ok(launchpad::get_rocket(config, wgstate))
    }

    #[test]
    fn add_peer_with_only_public_key() -> Result<(), failure::Error> {
        let db_file = mktemp::Temp::new_file()?;
        let rocket = get_test_rocket(db_file.to_path_buf())?;
        let client = Client::new(rocket)?;

        let public_key_input = "SwgTyJpz0og0NH/1YagZ2pWuaR06b0nlVUUo0WFdbAY=";

        let response = client
            .post("/peers/add")
            .header(ContentType::Form)
            .body(format!(
                "public_key={}",
                Uri::percent_encode(public_key_input),
            ))
            .dispatch();
        assert_eq!(response.status(), Status::Ok);

        let expected_public_key = PublicKey::from_str(public_key_input)?;

        let mut wg = WgSocket::connect()?;
        let device = wg.get_device(DeviceInterface::from_name("wgtest"))?;
        let peer = device
            .peers
            .iter()
            .find(|peer| &peer.public_key == expected_public_key.as_bytes());

        assert!(
            peer.is_some(),
            "Failed to find new peer in {:#?}",
            device.peers
        );

        Ok(())
    }

    #[test]
    fn add_peer_with_whitespace_preshared_key() -> Result<(), failure::Error> {
        let db_file = mktemp::Temp::new_file()?;
        let rocket = get_test_rocket(db_file.to_path_buf())?;
        let client = Client::new(rocket)?;

        let public_key_input = "8h7VPAMcU7MsDEdq2lvjYhsHOHxx2sM5L4GM4xZT5hQ=";

        let response = client
            .post("/peers/add")
            .header(ContentType::Form)
            .body(format!(
                "public_key={}&preshared_key={}",
                Uri::percent_encode(public_key_input),
                "   ",
            ))
            .dispatch();
        assert_eq!(response.status(), Status::Ok);

        let expected_public_key = PublicKey::from_str(public_key_input)?;

        let mut wg = WgSocket::connect()?;
        let device = wg.get_device(DeviceInterface::from_name("wgtest"))?;
        let peer = device
            .peers
            .iter()
            .find(|peer| &peer.public_key == expected_public_key.as_bytes())
            .expect("Newly added peer not found");

        assert_eq!(peer.preshared_key, [0u8; 32]);

        Ok(())
    }

    #[test]
    fn add_peer_all_fields() -> Result<(), failure::Error> {
        let db_file = mktemp::Temp::new_file()?;
        let rocket = get_test_rocket(db_file.to_path_buf())?;
        let client = Client::new(rocket)?;

        let public_key_input = "uQlHszU0iBTXja3UyIzt+lDVSPkDrmeeWWuEytox6jU=";
        let preshared_key_input = "CJizCOvSz4+S+PqG9XenDsBxRivLFPK3Hec9tQ3wEEU=";
        let allowed_ips_input = "10.0.0.1/32, 10.2.0.0/24";
        let endpoint_input = "192.168.1.101:51820";
        let persistent_keepalive_input = "10";

        let response = client
            .post("/peers/add")
            .header(ContentType::Form)
            .body(format!(
                "public_key={}&preshared_key={}&allowed_ips={}&endpoint={}&persistent_keepalive={}",
                Uri::percent_encode(public_key_input),
                Uri::percent_encode(preshared_key_input),
                Uri::percent_encode(allowed_ips_input),
                endpoint_input,
                persistent_keepalive_input,
            ))
            .dispatch();
        assert_eq!(response.status(), Status::Ok);

        let expected_public_key = PublicKey::from_str(public_key_input)?;
        let expected_preshared_key = PresharedKey::from_str(preshared_key_input)?;
        let expected_allowed_ips: Vec<get::AllowedIp> =
            (allowed_ips_input.parse::<AllowedIps>()?.0)
                .iter()
                .map(|allowed_ip| allowed_ip.into())
                .collect();
        let expected_endpoint: SocketAddr = endpoint_input.parse()?;
        let expected_persistent_keepalive: u16 = persistent_keepalive_input.parse()?;

        let mut wg = WgSocket::connect()?;
        let device = wg.get_device(DeviceInterface::from_name("wgtest"))?;
        let peer = device
            .peers
            .iter()
            .find(|peer| &peer.public_key == expected_public_key.as_bytes())
            .expect("Newly added peer not found");

        assert_eq!(&peer.preshared_key, expected_preshared_key.as_bytes());
        assert_eq!(peer.allowed_ips, expected_allowed_ips);
        assert_eq!(peer.endpoint, Some(expected_endpoint));
        assert_eq!(
            peer.persistent_keepalive_interval,
            expected_persistent_keepalive
        );

        Ok(())
    }
}
