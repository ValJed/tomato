use crate::structs::{App, CalendarSection, State};
use crate::utils;
use ratatui::crossterm::event::{KeyCode, KeyEvent};
use time::{Date, Duration, OffsetDateTime};

impl App {
  pub fn handle_calendar_input(&mut self, key_event: KeyEvent) {
    match key_event.code {
      KeyCode::Char('c') => {
        self.reset_calendar();
        self.state = State::None;
      }
      KeyCode::Char('h') | KeyCode::Left => {
        if let CalendarSection::Calendar = self.calendar.selected_section {
          self.prev_day()
        }
      }
      KeyCode::Char('j') | KeyCode::Down => {
        match self.calendar.selected_section {
          CalendarSection::Calendar => self.next_week(),
          CalendarSection::List => {
            self.select_next_session();
          }
        }
      }
      KeyCode::Char('k') | KeyCode::Up => {
        match self.calendar.selected_section {
          CalendarSection::Calendar => self.prev_week(),
          CalendarSection::List => {
            self.select_prev_session();
          }
        }
      }
      KeyCode::Char('l') | KeyCode::Right => {
        if let CalendarSection::Calendar = self.calendar.selected_section {
          self.next_day()
        }
      }
      KeyCode::Tab => self.switch_cal_section(),
      KeyCode::Char('p') => {
        self.reset_calendar();
        self.list_projects()
      }
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

  fn switch_cal_section(&mut self) {
    match self.calendar.selected_section {
      CalendarSection::Calendar => {
        self.calendar.list_state.select(Some(0));
        self.calendar.selected_section = CalendarSection::List
      }
      CalendarSection::List => {
        self.calendar.list_state.select(Some(0));
        self.calendar.selected_section = CalendarSection::Calendar
      }
    }
  }

  pub fn select_next_session(&mut self) {
    let i = match self.calendar.list_state.selected() {
      None => 0,
      Some(index) => {
        if index >= self.calendar.sessions.len() - 1 {
          0
        } else {
          index + 1
        }
      }
    };
    self.calendar.list_state.select(Some(i));
  }

  pub fn select_prev_session(&mut self) {
    let i = match self.calendar.list_state.selected() {
      None => 0,
      Some(index) => {
        if index == 0 {
          self.calendar.sessions.len() - 1
        } else {
          index - 1
        }
      }
    };
    self.calendar.list_state.select(Some(i));
  }

  pub fn reset_calendar(&mut self) {
    self.calendar.selected_date = None;
    self.calendar.selected_section = CalendarSection::Calendar;
    self.calendar.list_state.select(Some(0));
    self.calendar.sessions = vec![];
  }
}
