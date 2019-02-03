use diesel::connection::Connection;
use diesel::SqliteConnection;
use failure::{format_err, Error};
use rocket::config::Value;
use std::collections::HashMap;

// It'd be better to do "use diesel_migrations::embed_migrations" to be in line with the Rust 2018
// module changes, but something about this macro requires it to be imported using the old method
// in main.rs.
//
// https://github.com/diesel-rs/diesel/issues/1894
embed_migrations!();

fn connect(path: &str) -> Result<SqliteConnection, Error> {
    SqliteConnection::establish(&path).map_err(|_| format_err!("Unable to open db file: {}", path))
}

pub fn run_migrations(path: &str) -> Result<(), Error> {
    let db_conn = connect(path)?;
    embedded_migrations::run_with_output(&db_conn, &mut std::io::stdout())?;
    Ok(())
}

pub fn make_rocket_database_config(path: &str) -> HashMap<&str, Value> {
    // https://api.rocket.rs/v0.4/rocket_contrib/databases/index.html#procedurally
    let mut database_config = HashMap::new();
    let mut databases = HashMap::new();

    database_config.insert("url", Value::from(path));
    databases.insert("sqlite", Value::from(database_config));
    databases
}
