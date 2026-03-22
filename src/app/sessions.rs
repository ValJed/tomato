use crate::structs::{App, Session, SessionType, State};
use crate::utils;

impl App {
  pub fn start_work_input(&mut self) {
    if self.options.data.ask_before_work {
      self.input = String::from(self.options.data.work_duration.to_string());
      self.state = State::WorkInput
    } else {
      self.start_work_session()
    }
  }

  pub fn start_break_input(&mut self) {
    if self.options.data.ask_before_break {
      self.input = String::from(self.options.data.break_duration.to_string());
      self.state = State::BreakInput;
    } else {
      self.start_break_session();
    }
  }

  pub fn start_work_session(&mut self) {
    let time: u32 = self
      .input
      .parse()
      .unwrap_or(self.options.data.work_duration);
    self.current_session = Some(Session::new(SessionType::Work, time));
    self.state = State::WorkSession;
  }

  pub fn start_break_session(&mut self) {
    let time: u32 = self
      .input
      .parse()
      .unwrap_or(self.options.data.break_duration);
    self.current_session = Some(Session::new(SessionType::Break, time));
    self.state = State::BreakSession;
  }

  pub fn stop_work_session(&mut self) {
    let session = self.current_session.as_ref().unwrap();
    let spent_time = utils::get_spent_time(session.start, session.duration);

    if let Some(project_id) = self.get_selected_project().map(|p| p.id.clone())
    {
      let updated = self.repo.add_session(project_id, spent_time);
      if updated.is_err() {
        utils::notify("Error when updating project spent time");
      }
    }

    utils::notify("Break Time?");
    self.current_session = None;
  }

  pub fn stop_break_session(&mut self) {
    utils::notify("Back to work?");
    self.state = State::ConfirmWork;
    self.current_session = None;
  }

  pub fn toggle_session(&mut self) {
    match self.state {
      State::ConfirmBreak => {
        self.start_break_session();
      }
      State::WorkSession => {
        self.stop_work_session();
        self.state = State::ConfirmBreak;
      }
      State::BreakSession => {
        self.stop_break_session();
      }
      _ => self.start_work_input(),
    }
  }
}
