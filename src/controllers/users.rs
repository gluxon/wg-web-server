use failure::Error;
use rocket::{FromForm, post};
use rocket::http::Status;
use rocket::request::Form;
use crate::fairings::Database;
use crate::models::user;

#[derive(FromForm)]
pub struct CreateForm {
   email: String,
   password: String,
}

#[post("/", data = "<form>")]
pub fn create(conn: Database, form: Form<CreateForm>) -> Result<Status, Error> {
   user::insert(&conn, &user::NewUser {
      email: &form.email,
      password: &form.password,
   })?;
   Ok(Status::Ok)
}
