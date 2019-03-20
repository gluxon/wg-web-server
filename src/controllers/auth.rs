use crate::controllers::index::*;
use crate::fairings::Database;
use crate::models::User;
use askama::Template;
use failure;
use rocket::http::{Cookie, Cookies};
use rocket::request::Form;
use rocket::response::{Flash, Redirect};
use rocket::{get, post, uri, FromForm, Responder};

#[derive(Template)]
#[template(path = "auth/login.html")]
pub struct LoginTemplate {}

#[get("/login")]
pub fn login() -> LoginTemplate {
    LoginTemplate {}
}

#[derive(FromForm)]
pub struct LoginCredentials {
    email: String,
    password: String,
}

#[derive(Responder)]
pub enum PostLoginOk {
    Redirect(Redirect),
    FlashRedirect(Flash<Redirect>),
}

#[post("/login", data = "<credentials>")]
pub fn post_login(
    db: Database,
    mut cookies: Cookies,
    credentials: Form<LoginCredentials>,
) -> Result<PostLoginOk, failure::Error> {
    let user = User::by_email(&db, &credentials.email)?;

    let valid_credentials = user
        .as_ref()
        .map(|user| user.verify_password(&credentials.password))
        .unwrap_or(Ok(false))?;

    if let Some(user) = user.filter(|_| valid_credentials) {
        let cookie = Cookie::new("user_id", user.id.to_string());
        cookies.add_private(cookie);
        Ok(PostLoginOk::Redirect(Redirect::to(uri!(index))))
    } else {
        Ok(PostLoginOk::FlashRedirect(Flash::error(
            Redirect::to(uri!(login)),
            "Invalid username/password.",
        )))
    }
}

#[post("/logout")]
pub fn logout(mut cookies: Cookies) -> Flash<Redirect> {
    cookies.remove_private(Cookie::named("user_id"));
    Flash::success(Redirect::to(uri!(index)), "Successfully logged out")
}
