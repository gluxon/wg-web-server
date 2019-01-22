#![feature(proc_macro_hygiene, decl_macro)]

use rocket::routes;

mod asset;
mod controllers;

fn main() {
    rocket::ignite()
        .mount("/", asset::Asset)
        .mount("/", routes![controllers::index::index])
        .launch();
}
