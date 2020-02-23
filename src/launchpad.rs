use crate::asset;
use crate::cli;
use crate::controllers;
use crate::db;
use crate::fairings;
use crate::states;
use rocket::config::{Config, ConfigError, Environment};
use rocket::{routes, Rocket};

pub fn get_config_from_args(args: &cli::Args) -> Result<Config, ConfigError> {
    Config::build(Environment::active()?)
        .address(&args.bind_ip)
        .port(args.port)
        .extra("databases", db::make_rocket_database_config(&args.db_path))
        .finalize()
}

pub fn get_rocket(config: Config, wgstate: states::WgState) -> Rocket {
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
}
