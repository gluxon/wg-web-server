#![feature(proc_macro_hygiene, decl_macro)]

use exitfailure::ExitFailure;

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
mod launchpad;
mod models;
mod schema;
mod states;

fn main() -> Result<(), ExitFailure> {
    let args = cli::Args::get_from_clap()?;
    let interface_config =
        config::Config::init_from_path(args.interface.clone(), &args.interface_config)?;

    let should_daemonize = !args.foreground && !cfg!(debug_assertions);
    if should_daemonize {
        println!("Daemonizing will be supported in a later release.")
    }

    db::run_migrations(&args.db_path)?;

    let wgstate = states::WgState::init(interface_config)?;
    wgstate.apply_config()?;

    let config = launchpad::get_config_from_args(&args)?;
    launchpad::get_rocket(config, wgstate).launch();

    Ok(())
}
