#![feature(proc_macro_hygiene, decl_macro)]

use exitfailure::ExitFailure;
use rocket::routes;
use rocket::config::{Config, Environment};

mod asset;
mod cli;
mod controllers;

fn main() -> Result<(), ExitFailure> {
    let args = cli::Args::get_from_clap()?;

    let should_daemonize = !args.foreground && !cfg!(debug_assertions);
    if should_daemonize {
        println!("Daemonizing will be supported in a later release.")
    }

    let config = Config::build(Environment::active()?)
        .address(args.bind_ip)
        .port(args.port)
        .finalize()?;

    rocket::custom(config)
        .mount("/", asset::Asset)
        .mount("/", routes![controllers::index::index])
        .launch();

    Ok(())
}
