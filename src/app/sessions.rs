use crate::structs::{App, Project, Session, SessionType, State};
use crate::utils;
use tui_input::Input;

impl App {
  pub fn start_work_input(&mut self) {
    self.input = Input::new(self.default_work_duration.to_string());
    self.state = State::WorkInput
  }

  pub fn start_break_input(&mut self) {
    self.input = Input::new(self.default_break_duration.to_string());
    self.state = State::BreakInput;
  }

  pub fn start_work_session(&mut self) {
    let time: u32 = self
      .input
      .value()
      .parse()
      .unwrap_or(self.default_work_duration);
    self.state = State::WorkSession;
    self.current_session = Some(Session::new(SessionType::Work, time));
  }

  pub fn start_break_session(&mut self) {
    let time: u32 = self
      .input
      .value()
      .parse()
      .unwrap_or(self.default_break_duration);
    self.state = State::BreakSession;
    self.current_session = Some(Session::new(SessionType::Break, time));
  }

  pub fn stop_work_session(&mut self) {
    let session = self.current_session.as_ref().unwrap();
    let spent_time = utils::get_spent_time(session.start, session.duration);

    if let Some(project_id) = self.get_selected_project().map(|p| p.id.clone())
    {
      let updated = self.repo.add_session(project_id, spent_time);
      if updated.is_err() {
        let err = updated.unwrap();

        println!("err: {:?}", err);
        utils::notify("Error when updating project spent time");
      }
    }
  }

  pub fn toggle_session(&mut self) {
    if self.current_session.is_none() {
      if let State::ConfirmBreak = self.state {
        self.start_break_session();
      } else {
        self.start_work_session();
      }
      return;
    }

    match self.state {
      State::WorkSession => {
        self.stop_work_session();
        utils::notify("Break Time?");
        self.state = State::ConfirmBreak;
      }
      State::BreakSession => {
        utils::notify("Back to work?");
        self.state = State::ConfirmWork;
      }
      _ => {}
    }
    self.current_session = None;
  }
}
