use crate::structs::{App, State};
use ratatui::crossterm::event::{KeyCode, KeyEvent};

impl App {
  pub fn handler_options_input(&mut self, key_event: KeyEvent) {
    match key_event.code {
      KeyCode::Char('q') => self.exit(),
      KeyCode::Char('o') | KeyCode::Esc => {
        self.state = State::None;
      }
      KeyCode::Down | KeyCode::Char('j') => self.next_option(),
      KeyCode::Up | KeyCode::Char('k') => self.prev_option(),
      _ => {}
    }
  }

  pub fn prev_option(&mut self) {
    let new_index = self.options.selected_index - 1;
    let valid_index = if new_index == 0 {
      self.options.list.len() - 1
    } else {
      new_index
    };
    self.options.selected_index = valid_index;
  }

  pub fn next_option(&mut self) {
    let new_index = self.options.selected_index + 1;
    let valid_index = if new_index >= self.options.list.len() - 1 {
      0
    } else {
      new_index
    };
    self.options.selected_index = valid_index;
  }

  pub fn display_options(&mut self) {
    self.state = State::Options;
  }
}
