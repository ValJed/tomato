use crate::structs::{App, State};
use ratatui::crossterm::event::{KeyCode, KeyEvent};

impl App {
  pub fn handle_calendar_input(&mut self, key_event: KeyEvent) {
    match key_event.code {
      KeyCode::Char('c') => self.state = State::None,
      KeyCode::Char('h') | KeyCode::Left => {}
      KeyCode::Char('j') | KeyCode::Down => {}
      KeyCode::Char('k') | KeyCode::Up => {}
      KeyCode::Char('l') | KeyCode::Right => {}
      _ => {}
    }
  }

  pub fn display_calendar(&mut self) {
    self.state = State::Calendar;
  }
}
