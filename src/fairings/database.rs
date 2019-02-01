use rocket_contrib::database;
use rocket_contrib::databases::diesel;

#[database("sqlite")]
pub struct Database(diesel::SqliteConnection);
