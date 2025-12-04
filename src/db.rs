use crate::structs::UserConfig;
use rusqlite::{Connection, Result};

pub fn connect_db(config: &UserConfig) -> Result<Connection, rusqlite::Error> {
    // create missing folders in the path.
    let db = Connection::open(config.db_location.clone())?;

    db.execute(
        "CREATE TABLE project (
            id INTEGER PRIMARY KEY
            name TEXT NOT NULL
            time_spent INTEGER
            work_sessions: INTEGER
            creation_date TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            modification_date TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
        (),
    )?;

    Ok(db)
}
