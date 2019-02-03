use askama::Template;
use rocket::get;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate<'a> {
    name: &'a str,
}

#[get("/")]
pub fn index() -> IndexTemplate<'static> {
    IndexTemplate { name: "world" }
}
