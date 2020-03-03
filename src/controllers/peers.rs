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
