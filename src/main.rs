#![feature(proc_macro_hygiene, decl_macro)]

use exitfailure::ExitFailure;
use rocket::config::{Config, Environment};
use rocket::routes;

// https://github.com/diesel-rs/diesel/issues/1894#issuecomment-433178841
#[macro_use]
extern crate diesel_migrations;

mod asset;
mod cli;
mod controllers;
mod db;
mod fairings;
mod net;

fn main() -> Result<(), ExitFailure> {
    let args = cli::Args::get_from_clap()?;
    let interface = net::Interface::init_from_path(&args.interface_config)?;

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

    rocket::custom(config)
        .attach(fairings::Database::fairing())
        .mount("/", asset::Asset)
        .mount("/", routes![controllers::index::index])
        .launch();

    Ok(())
}
