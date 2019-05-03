use crate::diesel;
use crate::schema::peers;
use diesel::prelude::*;
use failure::Error;

#[derive(diesel::Queryable)]
pub struct Peer {
    pub id: i32,
    pub public_key: String,
}

impl Peer {
    pub fn by_public_key(conn: &SqliteConnection, public_key: &str) -> QueryResult<Option<Self>> {
        match peers::table.filter(peers::public_key.eq(public_key)).first(conn) {
            Ok(peer) => Ok(Some(peer)),
            Err(diesel::result::Error::NotFound) => Ok(None),
            Err(err) => Err(err),
        }
    }
}

#[derive(diesel::Insertable)]
#[table_name = "peers"]
pub struct NewPeer<'a> {
    pub public_key: &'a str,
}

pub fn insert(conn: &SqliteConnection, new_peer: &NewPeer) -> Result<(), Error> {
    diesel::insert_into(peers::table)
        .values(new_peer)
        .execute(conn)?;

    Ok(())
}
