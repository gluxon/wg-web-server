use crate::diesel;
use crate::schema::users;
use argon2;
use diesel::prelude::*;
use failure::Error;
use rand_os;
use rand_os::rand_core::RngCore;

fn hash(text: &str) -> Result<String, Error> {
    let mut salt = [0u8; 16];
    let mut os_rng = rand_os::OsRng::new()?;
    os_rng.fill_bytes(&mut salt);

    let mut config = argon2::Config::default();
    config.variant = argon2::Variant::Argon2id;

    let hash = argon2::hash_encoded(text.as_bytes(), &salt, &config)?;
    Ok(hash)
}

#[derive(diesel::Queryable)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub password: Option<String>,
    pub administrator: i32,
}

impl User {
    pub fn by_email(conn: &SqliteConnection, email: &str) -> QueryResult<Option<Self>> {
        match users::table.filter(users::email.eq(email)).first(conn) {
            Ok(user) => Ok(Some(user)),
            Err(diesel::result::Error::NotFound) => Ok(None),
            Err(err) => Err(err),
        }
    }

    pub fn verify_password(&self, password: &str) -> argon2::Result<bool> {
        match self.password {
            None => Ok(false),
            Some(ref hash) => argon2::verify_encoded(hash, password.as_bytes()),
        }
    }
}

#[derive(diesel::Insertable)]
#[table_name = "users"]
pub struct NewUser<'a> {
    pub email: &'a str,
    pub password: &'a str,
}

pub fn insert(conn: &SqliteConnection, new_user: &NewUser) -> Result<(), Error> {
    let new_user = NewUser {
        email: new_user.email,
        password: &hash(new_user.password)?,
    };

    diesel::insert_into(users::table)
        .values(&new_user)
        .execute(conn)?;

    Ok(())
}
