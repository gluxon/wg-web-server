use crate::config::publickey::PublicKey;
use crate::states::WgState;
use askama::Template;
use failure;
use rocket::http::Status;
use rocket::request::Form;
use rocket::{get, State};
use rocket::{post, FromForm};
use std::str::FromStr;

#[derive(Template)]
#[template(path = "peers/add.html")]
pub struct AddPeerTemplate {}

#[get("/add")]
pub fn add() -> AddPeerTemplate {
    AddPeerTemplate {}
}

#[derive(FromForm)]
pub struct AddForm {
    public_key: String,
}

#[post("/", data = "<form>")]
pub fn post_add(wg: State<WgState>, form: Form<AddForm>) -> Result<Status, failure::Error> {
    // TODO:
    //   - Record this new peer in the database.
    //   - Calculate the next available IP and give it to this peer.
    //   - Validate the form and return a 400 status code if it has errors.

    let public_key = PublicKey::from_str(&form.public_key)?;
    wg.add_peer(public_key.as_bytes())?;
    Ok(Status::Ok)
}
