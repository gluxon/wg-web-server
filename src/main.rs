#![feature(proc_macro_hygiene, decl_macro)]

use rocket::routes;
use rocket_contrib::serve::StaticFiles;

mod controllers;

fn main() {
    rocket::ignite()
        .mount("/", routes![controllers::index])
        .mount("/", StaticFiles::from("static"))
        .launch();
}
