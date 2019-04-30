use crate::fairings::Database;
use crate::models::user;
use failure::Error;
use rocket::http::Status;
use rocket::request::Form;
use rocket::{post, FromForm};

#[derive(FromForm)]
pub struct CreateForm {
    email: String,
    password: String,
}

#[post("/", data = "<form>")]
pub fn create(conn: Database, form: Form<CreateForm>) -> Result<Status, Error> {
    user::insert(
        &conn,
        &user::NewUser {
            email: &form.email,
            password: &form.password,
        },
    )?;
    Ok(Status::Ok)
}
