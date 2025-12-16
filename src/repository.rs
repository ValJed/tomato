use crate::structs::{Project, UserConfig};
use rusqlite::{Connection, Result};
use std::error::Error;
use std::fs::create_dir_all;
use std::path::Path;

pub struct ProjectRepository {
  connection: Connection,
}

impl ProjectRepository {
  pub fn new(config: &UserConfig) -> Result<Self, Box<dyn Error>> {
    let db_location = config.db_location.clone();
    let (db_folder_path, _file) = db_location
      .rsplit_once("/")
      .ok_or("DB location isn't valid")?;

    if !Path::new(db_folder_path).exists() {
      create_dir_all(db_folder_path)?;
    }

    let connection = Connection::open(config.db_location.clone())
      .map_err(|e| e.to_string())?;

    connection.execute(
      "CREATE TABLE IF NOT EXISTS project (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                selected BOOLEAN NOT NULL DEFAULT FALSE,
                time_spent INTEGER NOT NULL DEFAULT 0,
                work_sessions INTEGER NOT NULL DEFAULT 0,
                creation_date TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                modification_date TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            )",
      (),
    )?;

    Ok(Self { connection })
  }

  pub fn get_all(&self) -> Result<Vec<Project>, rusqlite::Error> {
    let mut stmt = self
      .connection
      .prepare("SELECT * FROM project ORDER BY project.id ASC")?;

    let projects = stmt
      .query_map([], |row| {
        Ok(Project {
          id: row.get(0)?,
          name: row.get(1)?,
          selected: row.get(2)?,
          time_spent: row.get(3)?,
          work_sessions: row.get(4)?,
          creation_date: row.get(5)?,
          modification_date: row.get(6)?,
        })
      })?
      .collect::<Result<Vec<_>, _>>()?;

    Ok(projects)
  }

  pub fn add(&self, name: String) -> Result<(), rusqlite::Error> {
    self
      .connection
      .execute("INSERT INTO project (name) VALUES (?1)", [&name])?;
    Ok(())
  }

  pub fn update(&self, id: usize, name: String) -> Result<(), rusqlite::Error> {
    self.connection.execute(
            "UPDATE project SET name = ?1, modification_date = CURRENT_TIMESTAMP WHERE id = ?2",
            (name, id),
        )?;
    Ok(())
  }

  pub fn delete(&self, id: usize) -> Result<(), rusqlite::Error> {
    self
      .connection
      .execute("DELETE FROM project WHERE id = ?1", [&id.to_string()])?;
    Ok(())
  }

  pub fn update_project_time(
    &self,
    id: usize,
    time: u32,
  ) -> Result<(), rusqlite::Error> {
    self.connection.execute(
      "UPDATE project SET time_spent = time_spent + ?1, 
        modification_date = CURRENT_TIMESTAMP 
        WHERE id = ?2",
      (time, id),
    )?;
    Ok(())
  }

  pub fn set_selected(
    &self,
    id: usize,
    selected: bool,
  ) -> Result<(), rusqlite::Error> {
    if selected == true {
      self.connection.execute(
                "UPDATE project SET selected = 0, modification_date = CURRENT_TIMESTAMP WHERE selected = 1",
                []
            )?;
    }
    let selected_num = if selected == true { "1" } else { "0" };
    self.connection.execute(
            "UPDATE project SET selected = ?1, modification_date = CURRENT_TIMESTAMP WHERE id = ?2",
            [selected_num, &id.to_string()],
        )?;
    Ok(())
  }

  pub fn get_by_id(&self, id: i32) -> Result<Option<Project>, rusqlite::Error> {
    let mut stmt = self
      .connection
      .prepare("SELECT * FROM project WHERE id = ?1")?;

    let mut rows = stmt.query([&id.to_string()])?;

    if let Some(row) = rows.next()? {
      Ok(Some(Project {
        id: row.get(0)?,
        name: row.get(1)?,
        selected: row.get(2)?,
        time_spent: row.get(3)?,
        work_sessions: row.get(4)?,
        creation_date: row.get(5)?,
        modification_date: row.get(6)?,
      }))
    } else {
      Ok(None)
    }
  }
}
