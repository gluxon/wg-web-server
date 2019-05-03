use crate::states::WgState;
use crate::fairings::Database;
use crate::models::peer;
use base64;
use failure::Error;
use rocket::http::Status;
use rocket::request::Form;
use rocket::State;
use rocket::{post, FromForm, Responder};
use rocket_contrib::json::Json;
use serde::Serialize;
use std::net::Ipv4Addr;

#[derive(FromForm)]
pub struct CreateForm {
    public_key: String,
}

#[derive(Responder)]
pub enum CreateResponse {
    Status(Status),
    Json(Json<CreateResponseJson>),
}

#[derive(Serialize)]
pub struct CreateResponseJson {
    address: Ipv4Addr,
}

#[post("/", data = "<form>")]
pub fn create(
    conn: Database,
    wg: State<WgState>,
    form: Form<CreateForm>
) -> Result<CreateResponse, Error> {
    peer::insert(
        &conn,
        &peer::NewPeer {
            public_key: &form.public_key,
        },
    )?;

    let peer = match peer::Peer::by_public_key(&conn, &form.public_key)? {
        Some(peer) => peer,
        None => return Ok(CreateResponse::Status(Status::InternalServerError)),
    };

    if peer.id > i32::from(std::u16::MAX) {
        return Ok(CreateResponse::Status(Status::InternalServerError));
    }

    let ipaddr = {
        let [c, d] = (peer.id as u16).to_be_bytes();
        Ipv4Addr::new(10, 24, c, d)
    };

    let mut public_key = [0u8; 32];
    public_key.copy_from_slice(&base64::decode(&form.public_key)?);
    wg.add_peer(public_key, ipaddr.clone())?;

    Ok(CreateResponse::Json(rocket_contrib::json::Json(CreateResponseJson {
        address: ipaddr,
    })))
}
