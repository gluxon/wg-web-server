use diesel::SqliteConnection;
use diesel::connection::Connection;
use failure::{Error, format_err};

// It'd be better to do "use diesel_migrations::embed_migrations" to be in line with the Rust 2018
// module changes, but something about this macro requires it to be imported using the old method
// in main.rs.
//
// https://github.com/diesel-rs/diesel/issues/1894
embed_migrations!();

fn connect(path: &str) -> Result<SqliteConnection, Error> {
    SqliteConnection::establish(&path)
        .map_err(|_| format_err!("Unable to open db file: {}", path))
}

pub fn run_migrations(path: &str) -> Result<(), Error> {
    let db_conn = connect(path)?;
    embedded_migrations::run_with_output(&db_conn, &mut std::io::stdout())?;
    Ok(())
}
