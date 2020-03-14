use crate::config::publickey::PublicKey;
use crate::lang;
use crate::states::WgState;
use crate::utils::FormInputResult;
use askama::Template;
use rocket::http::Status;
use rocket::request::Form;
use rocket::response::status;
use rocket::{get, State};
use rocket::{post, FromForm};
use std::borrow::Cow;
use std::default::Default;

#[derive(Default, Template)]
#[template(path = "peers/add.html")]
pub struct AddPeerTemplate<'a> {
    status: Option<Cow<'a, str>>,
    public_key_err: Option<String>,
}

#[get("/add")]
pub fn add() -> AddPeerTemplate<'static> {
    AddPeerTemplate::default()
}

#[derive(FromForm)]
pub struct AddPeer<'v> {
    public_key: FormInputResult<'v, PublicKey>,
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
            println!("{}", public_key_err.input);
            let template = AddPeerTemplate {
                public_key_err: Some(format!("{}", public_key_err.error)),
                ..Default::default()
            };
            return status::Custom(Status::BadRequest, template);
        }
    };

    let add_peer_result = wg.add_peer(public_key.as_bytes());
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
    use crate::config::PublicKey;
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
    use std::path::PathBuf;
    use std::str::FromStr;
    use wireguard_uapi::{DeviceInterface, WgSocket};

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
    fn add_peer() -> Result<(), failure::Error> {
        let db_file = mktemp::Temp::new_file()?;
        let rocket = get_test_rocket(db_file.to_path_buf())?;
        let client = Client::new(rocket)?;

        let public_key_base64 = "SwgTyJpz0og0NH/1YagZ2pWuaR06b0nlVUUo0WFdbAY=";
        let public_key = PublicKey::from_str(public_key_base64)?;

        let response = client
            .post("/peers/add")
            .header(ContentType::parse_flexible("application/x-www-form-urlencoded").unwrap())
            .body(format!(
                "public_key={}",
                Uri::percent_encode(public_key_base64)
            ))
            .dispatch();
        assert_eq!(response.status(), Status::Ok);

        let mut wg = WgSocket::connect()?;
        let device = wg.get_device(DeviceInterface::from_name("wgtest"))?;
        assert!(device
            .peers
            .iter()
            .any(|peer| &peer.public_key == public_key.as_bytes()));

        Ok(())
    }
}
