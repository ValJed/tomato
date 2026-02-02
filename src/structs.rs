use dirs::data_dir;
use ratatui::widgets::ListState;
use serde::{Deserialize, Serialize};
use std::default::Default;
use std::time::SystemTime;
use time::Date;

use crate::repository::ProjectRepository;

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct UserConfig {
  pub default_work_duration: u32,
  pub default_break_duration: u32,
  pub db_location: String,
}

impl Default for UserConfig {
  fn default() -> Self {
    let mut db_path = data_dir().expect("Could not find local data directory");
    db_path.push("tomato/tomato.sqlite");
    let db_location = match db_path.into_os_string().into_string() {
      Ok(path) => path,
      Err(_) => String::new(),
    };

    Self {
      default_work_duration: 25,
      default_break_duration: 5,
      db_location,
    }
  }
}

pub struct App {
  pub state: State,
  pub exit: bool,
  pub current_session: Option<Session>,
  pub input: String,
  pub repo: ProjectRepository,
  pub projects_list: ProjectsList,
  pub calendar: CalendarState,
  pub default_work_duration: u32,
  pub default_break_duration: u32,
}

#[derive(Default)]
pub struct ProjectsList {
  pub projects: Vec<Project>,
  pub selected_id: Option<usize>,
  pub state: ListState,
}

#[derive(Default)]
pub enum InputMode {
  #[default]
  Normal,
  Editing,
}

#[derive(Debug)]
pub enum CalendarSection {
  Calendar,
  List,
}

impl Default for CalendarSection {
  fn default() -> Self {
    CalendarSection::Calendar
  }
}

#[derive(Default)]
pub struct CalendarState {
  pub selected_date: Option<Date>,
  pub sessions: Vec<SessionPerDay>,
  pub list_state: ListState,
  pub selected_section: CalendarSection,
}

#[derive(Copy, Clone, Debug)]
pub enum SessionType {
  Work,
  Break,
}

#[derive(Debug)]
pub enum State {
  None,
  WorkSession,
  BreakSession,
  ConfirmBreak,
  ConfirmWork,
  ConfirmFinished,
  ChooseTime,
  WorkInput,
  BreakInput,
  ProjectsList,
  ProjectsInputAdd,
  ProjectsInputUpdate,
  Calendar,
}

impl Default for State {
  fn default() -> Self {
    State::None
  }
}

pub struct Session {
  pub start: SystemTime,
  pub end: Option<SystemTime>,
  pub duration: u32,
  pub session_type: SessionType,
}

#[derive(Debug)]
pub struct SessionPerDay {
  pub project_name: String,
  pub date: Date,
  pub duration: u32,
}

impl Session {
  pub fn new(session_type: SessionType, duration: u32) -> Self {
    Self {
      start: SystemTime::now(),
      end: None,
      duration,
      session_type,
    }
  }
}

#[derive(Debug, Clone)]
pub struct Project {
  pub id: usize,
  pub name: String,
  pub selected: bool,
  pub time_spent: i32,
  pub work_sessions: i32,
  pub finished: bool,
  pub creation_date: String,
  pub modification_date: String,
}
