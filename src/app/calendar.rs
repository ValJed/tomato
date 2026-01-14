use crate::structs::{App, SessionPerDay, State};
use crate::utils;
use ratatui::crossterm::event::{KeyCode, KeyEvent};
use time::{Date, Duration, OffsetDateTime};

impl App {
  pub fn handle_calendar_input(&mut self, key_event: KeyEvent) {
    match key_event.code {
      KeyCode::Char('c') => {
        self.calendar.selected_date = None;
        self.calendar.sessions = vec![];
        self.state = State::None;
      }
      KeyCode::Char('h') | KeyCode::Left => self.prev_day(),
      KeyCode::Char('j') | KeyCode::Down => self.next_week(),
      KeyCode::Char('k') | KeyCode::Up => self.prev_week(),
      KeyCode::Char('l') | KeyCode::Right => self.next_day(),
      KeyCode::Char('q') => self.exit(),
      _ => {}
    }
  }

  pub fn prev_day(&mut self) {
    match OffsetDateTime::now_local() {
      Ok(offset) => {
        let current = self.calendar.selected_date.unwrap_or(offset.date());
        self.calendar.selected_date = current.checked_sub(Duration::DAY);
        self.set_date_and_sessions(self.calendar.selected_date);
      }
      Err(_) => {
        self.calendar.selected_date = None;
      }
    }
  }

  pub fn next_day(&mut self) {
    match OffsetDateTime::now_local() {
      Ok(offset) => {
        let current = self.calendar.selected_date.unwrap_or(offset.date());
        self.calendar.selected_date = current.checked_add(Duration::DAY);
        self.set_date_and_sessions(self.calendar.selected_date);
      }
      Err(_) => {
        self.calendar.selected_date = None;
      }
    }
  }

  pub fn prev_week(&mut self) {
    match OffsetDateTime::now_local() {
      Ok(offset) => {
        let current = self.calendar.selected_date.unwrap_or(offset.date());
        self.calendar.selected_date = current.checked_sub(Duration::WEEK);
        self.set_date_and_sessions(self.calendar.selected_date);
      }
      Err(_) => {
        self.calendar.selected_date = None;
      }
    }
  }

  pub fn next_week(&mut self) {
    match OffsetDateTime::now_local() {
      Ok(offset) => {
        let current = self.calendar.selected_date.unwrap_or(offset.date());
        self.calendar.selected_date = current.checked_add(Duration::WEEK);
        self.set_date_and_sessions(self.calendar.selected_date);
      }
      Err(_) => {
        self.calendar.selected_date = None;
      }
    }
  }

  pub fn display_calendar(&mut self) {
    let cur_date = self
      .calendar
      .selected_date
      .unwrap_or(OffsetDateTime::now_local().unwrap().date());
    self.set_date_and_sessions(Some(cur_date));
    self.state = State::Calendar;
  }

  fn set_date_and_sessions(&mut self, date: Option<Date>) {
    self.calendar.selected_date = date;
    if date.is_none() {
      self.calendar.sessions = vec![];
    }

    match self.repo.get_sessions_per_day(&date.unwrap()) {
      Ok(sessions) => self.calendar.sessions = sessions,
      Err(err) => {
        utils::notify(&err.to_string());
        self.calendar.sessions = vec![]
      }
    }
  }
}
