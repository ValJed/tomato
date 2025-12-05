use crate::structs::UserConfig;
use rusqlite::{Connection, Result};
use std::error::Error;
use std::fs::create_dir_all;
use std::path::Path;

pub fn connect_db(config: &UserConfig) -> Result<Connection, Box<dyn Error>> {
    // create missing folders in the path.
    let db_location = config.db_location.clone();
    let (db_folder_path, _file) = db_location
        .rsplit_once("/")
        .ok_or("DB location isn't valid")?;

    if !Path::new(db_folder_path).exists() {
        create_dir_all(db_folder_path)?;
    }

    let db = Connection::open(config.db_location.clone()).map_err(|e| e.to_string())?;

    Ok(db)
}
