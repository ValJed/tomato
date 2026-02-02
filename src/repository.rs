use crate::structs::{Options, Project, SessionPerDay, UserConfig};
use rusqlite::{Connection, Result};
use std::error::Error;
use std::fs::create_dir_all;
use std::path::Path;

use time::Date;

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
            finished BOOLEAN NOT NULL DEFAULT FALSE,
            creation_date TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            modification_date TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );",
      (),
    )?;

    connection.execute(
      "CREATE TABLE IF NOT EXISTS session (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            project_id INTEGER NOT NULL,
            date TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            duration INTEGER NOT NULL DEFAULT 0
        );",
      (),
    )?;

    connection.execute(
      "CREATE TABLE IF NOT EXISTS options (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            work_duration INTEGER DEFAULT 25,
            break_duration INTEGER DEFAULT 5,
            ask_before_work BOOLEAN NOT NULL DEFAULT FALSE,
            ask_before_break BOOLEAN NOT NULL DEFAULT TRUE
        );",
      (),
    )?;

    Ok(Self { connection })
  }

  pub fn create_of_get_options(&self) -> Result<Options, rusqlite::Error> {
    self.connection.execute(
      "INSERT OR IGNORE INTO options (
            id,
            work_duration, 
            break_duration, 
            ask_before_work, 
            ask_before_break
        ) 
        VALUES (1, 25, 5, false, false)",
      (),
    )?;

    self
      .connection
      .query_row("SELECT * FROM options LIMIT 1", [], |row| {
        Ok(Options {
          id: row.get(0)?,
          work_duration: row.get(1)?,
          break_duration: row.get(2)?,
          ask_before_work: row.get(3)?,
          ask_before_break: row.get(4)?,
        })
      })
  }

  pub fn get_projects_in_progress(
    &self,
  ) -> Result<Vec<Project>, rusqlite::Error> {
    let mut stmt = self.connection.prepare(
      "SELECT * FROM project WHERE finished = false ORDER BY project.id ASC",
    )?;

    stmt
      .query_map([], |row| {
        Ok(Project {
          id: row.get(0)?,
          name: row.get(1)?,
          selected: row.get(2)?,
          time_spent: row.get(3)?,
          work_sessions: row.get(4)?,
          finished: row.get(5)?,
          creation_date: row.get(6)?,
          modification_date: row.get(7)?,
        })
      })?
      .collect::<Result<Vec<_>, _>>()
  }

  pub fn add_project(&self, name: &str) -> Result<(), rusqlite::Error> {
    self
      .connection
      .execute("INSERT INTO project (name) VALUES (?1)", [name])?;
    Ok(())
  }

  pub fn update_project(
    &self,
    id: usize,
    name: &str,
  ) -> Result<(), rusqlite::Error> {
    self.connection.execute(
            "UPDATE project SET name = ?1, modification_date = CURRENT_TIMESTAMP WHERE id = ?2",
            (name, id),
        )?;
    Ok(())
  }

  pub fn mark_project_finished(
    &mut self,
    id: usize,
  ) -> Result<(), rusqlite::Error> {
    self.connection.execute(
      "UPDATE project SET finished = true WHERE id = ?1",
      [&id.to_string()],
    )?;
    Ok(())
  }

  pub fn add_session(
    &mut self,
    project_id: usize,
    duration: u32,
  ) -> Result<(), rusqlite::Error> {
    let tx = self.connection.transaction()?;
    let dur = duration as usize;
    tx.execute(
      "INSERT INTO session (project_id, duration) VALUES (?1, ?2);",
      [&project_id, &dur],
    )?;
    tx.execute(
      "UPDATE project SET time_spent = time_spent + ?1, 
        work_sessions = work_sessions + 1,
        modification_date = CURRENT_TIMESTAMP 
        WHERE id = ?2",
      (&dur, &project_id),
    )?;

    tx.commit()
  }

  pub fn set_selected(
    &mut self,
    id: usize,
    selected: bool,
  ) -> Result<(), rusqlite::Error> {
    let tx = self.connection.transaction()?;
    if selected == true {
      tx.execute(
        "UPDATE project SET selected = 0, 
        modification_date = CURRENT_TIMESTAMP 
        WHERE selected = 1",
        [],
      )?;
    }
    let selected_num = if selected == true { "1" } else { "0" };
    tx.execute(
      "UPDATE project SET selected = ?1, 
        modification_date = CURRENT_TIMESTAMP 
        WHERE id = ?2",
      [selected_num, &id.to_string()],
    )?;

    tx.commit()
  }

  pub fn get_project_by_id(
    &self,
    id: i32,
  ) -> Result<Option<Project>, rusqlite::Error> {
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
        finished: row.get(5)?,
        creation_date: row.get(6)?,
        modification_date: row.get(7)?,
      }))
    } else {
      Ok(None)
    }
  }

  pub fn get_sessions_per_day(
    &self,
    date: &Date,
  ) -> Result<Vec<SessionPerDay>, rusqlite::Error> {
    let request = r#"
      SELECT project.name AS project_name, DATE(session.date) as date, SUM(duration) AS duration
      FROM session 
      INNER JOIN project ON session.project_id = project.id
      WHERE DATE(session.date) = DATE(?1)
      GROUP BY project_id
    "#;
    let mut stmt = self.connection.prepare(request)?;

    stmt
      .query_map([date], |row| {
        Ok(SessionPerDay {
          project_name: row.get(0)?,
          date: row.get(1)?,
          duration: row.get(2)?,
        })
      })?
      .collect::<Result<Vec<SessionPerDay>, _>>()
  }
}
