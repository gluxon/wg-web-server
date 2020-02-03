#![feature(proc_macro_hygiene, decl_macro)]

use exitfailure::ExitFailure;
use rocket::config::{Config, Environment};
use rocket::routes;

// https://github.com/diesel-rs/diesel/issues/1894#issuecomment-433178841
#[macro_use]
extern crate diesel_migrations;
#[macro_use]
extern crate diesel;

mod asset;
mod cli;
mod config;
mod controllers;
mod db;
mod fairings;
mod models;
mod schema;
mod states;

fn main() -> Result<(), ExitFailure> {
    let args = cli::Args::get_from_clap()?;
    // TODO: Read and apply device & peers from the interface configuration object.
    let _interface = config::Interface::init_from_path(&args.interface_config)?;

    let should_daemonize = !args.foreground && !cfg!(debug_assertions);
    if should_daemonize {
        println!("Daemonizing will be supported in a later release.")
    }

    db::run_migrations(&args.db_path)?;

    let config = Config::build(Environment::active()?)
        .address(args.bind_ip)
        .port(args.port)
        .extra("databases", db::make_rocket_database_config(&args.db_path))
        .finalize()?;

    let wgstate = states::WgState::init(args.interface.clone())?;

    rocket::custom(config)
        .attach(fairings::Database::fairing())
        .manage(wgstate)
        .mount("/", asset::Asset)
        .mount("/", routes![controllers::index::index])
        .mount(
            "/auth",
            routes![
                controllers::auth::login,
                controllers::auth::post_login,
                controllers::auth::logout,
            ],
        )
        .mount("/network", routes![controllers::network::index,])
        .mount("/users", routes![controllers::users::create,])
        .launch();

    Ok(())
}
