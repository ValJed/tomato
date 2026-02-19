use crate::structs::{App, State};
use crate::utils::convert_bool_to_string;
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
      KeyCode::Char(' ') | KeyCode::Enter => self.update_option(),
      _ => {}
    }
  }

  pub fn prev_option(&mut self) {
    let valid_index = if self.options.selected_index == 0 {
      self.options.options_number - 1
    } else {
      self.options.selected_index - 1
    };
    self.options.selected_index = valid_index;
  }

  pub fn next_option(&mut self) {
    let valid_index =
      if self.options.selected_index >= self.options.options_number - 1 {
        0
      } else {
        self.options.selected_index + 1
      };
    self.options.selected_index = valid_index;
  }

  pub fn display_options(&mut self) {
    self.state = State::Options;
  }

  pub fn update_option(&mut self) {
    // let cur_option = self.options.se
  }
}

#[derive(Debug, Clone)]
pub struct OptionsState {
  pub data: Options,
  pub selected_index: usize,
  pub options_number: usize,
}

#[derive(Debug, Clone)]
pub struct Options {
  pub id: u32,
  pub work_duration: u32,
  pub break_duration: u32,
  pub ask_before_work: bool,
  pub ask_before_break: bool,
}

// UI navigation enum
#[derive(Debug, Clone, Copy)]
pub enum OptionField {
  WorkDuration,
  BreakDuration,
  AskBeforeWork,
  AskBeforeBreak,
}

pub enum BoolOrInt {
  Bool(bool),
  Int(u32),
}

impl Options {
  pub fn get_list(&self) -> [(OptionField, String, String); 4] {
    [
      (
        OptionField::WorkDuration,
        String::from("Work duration"),
        self.work_duration.to_string(),
      ),
      (
        OptionField::BreakDuration,
        String::from("Break duration"),
        self.break_duration.to_string(),
      ),
      (
        OptionField::AskBeforeWork,
        String::from("Ask time before work session"),
        convert_bool_to_string(self.ask_before_work),
      ),
      (
        OptionField::AskBeforeBreak,
        String::from("Ask time before break session"),
        convert_bool_to_string(self.ask_before_break),
      ),
    ]
  }

  pub fn get_value(&self, value: &OptionField) -> BoolOrInt {
    match value {
      OptionField::WorkDuration => BoolOrInt::Int(self.work_duration),
      OptionField::BreakDuration => BoolOrInt::Int(self.break_duration),
      OptionField::AskBeforeWork => BoolOrInt::Bool(self.ask_before_work),
      OptionField::AskBeforeBreak => BoolOrInt::Bool(self.ask_before_break),
    }
  }
}

// Bridge between enum and struct — read a value
