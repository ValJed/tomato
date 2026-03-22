use crate::App;
use crate::structs::State;
use crate::utils;
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
    let list = self.options.data.get_list();
    let Some((_, option)) = list
      .iter()
      .enumerate()
      .find(|(i, _)| *i == self.options.selected_index)
    else {
      return;
    };

    let cur_value = self.options.data.get_value(option.0);
    match cur_value {
      BoolOrInt::Int(_) => match option.0 {
        OptionField::WorkDuration => {
          self.input = self.options.data.work_duration.to_string();
          self.state = State::WorkDurationInput;
        }
        OptionField::BreakDuration => {
          self.input = self.options.data.break_duration.to_string();
          self.state = State::BreakDurationInput;
        }
        _ => {}
      },
      BoolOrInt::Bool(val) => {
        self.options.data.set_value(option.0, BoolOrInt::Bool(!val));
        if self.repo.update_options(self.options.data.clone()).is_err() {
          utils::notify("Error saving option");
        }
      }
    }
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

#[derive(Debug)]
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

  pub fn get_value(&self, field: OptionField) -> BoolOrInt {
    match field {
      OptionField::WorkDuration => BoolOrInt::Int(self.work_duration),
      OptionField::BreakDuration => BoolOrInt::Int(self.break_duration),
      OptionField::AskBeforeWork => BoolOrInt::Bool(self.ask_before_work),
      OptionField::AskBeforeBreak => BoolOrInt::Bool(self.ask_before_break),
    }
  }

  pub fn set_value(&mut self, field: OptionField, value: BoolOrInt) {
    match field {
      OptionField::WorkDuration => {
        if let BoolOrInt::Int(v) = value {
          self.work_duration = v;
        }
      }
      OptionField::BreakDuration => {
        if let BoolOrInt::Int(v) = value {
          self.break_duration = v;
        }
      }
      OptionField::AskBeforeWork => {
        if let BoolOrInt::Bool(v) = value {
          self.ask_before_work = v;
        }
      }
      OptionField::AskBeforeBreak => {
        if let BoolOrInt::Bool(v) = value {
          self.ask_before_break = v;
        }
      }
    };
  }
}
