use crate::structs::{App, Project, State};
use crate::utils;
use ratatui::crossterm::event::{KeyCode, KeyEvent};

impl App {
  pub fn handle_projects_list_input(&mut self, key_event: KeyEvent) {
    match key_event.code {
      KeyCode::Char('q') => self.exit(),
      KeyCode::Down | KeyCode::Char('j') => self.next_project(),
      KeyCode::Up | KeyCode::Char('k') => self.prev_project(),
      KeyCode::Char('p') | KeyCode::Esc => {
        self.state = State::None;
      }
      KeyCode::Char('a') => {
        self.state = State::ProjectsInputAdd;
      }
      KeyCode::Char('f') => {
        self.state = State::ConfirmFinished;
      }
      KeyCode::Char('u') => {
        let value = match self.get_highlighted_project() {
          Some(project) => project.name.clone(),
          None => String::new(),
        };

        self.input = String::from(value);
        self.state = State::ProjectsInputUpdate;
      }
      KeyCode::Char('c') => {
        self.display_calendar();
      }
      KeyCode::Char(' ') => {
        let selected_index = self.projects_list.state.selected();
        match selected_index {
          Some(index) => match self.projects_list.projects.get(index) {
            Some(project) => {
              self.set_selected_project(project.id);
            }
            None => {}
          },
          _ => {}
        }
      }
      _ => {}
    }
  }

  pub fn handle_project_input(&mut self, key_event: KeyEvent) {
    match key_event.code {
      KeyCode::Char(char) => {
        let char_str = char.to_string();
        let new_input = self.input.to_string() + char_str.as_str();
        self.input = String::from(new_input);
      }
      KeyCode::Backspace => {
        let mut chars = self.input.chars();
        chars.next_back();
        self.input = chars.as_str().into();
      }
      KeyCode::Enter => match self.state {
        State::ProjectsInputAdd => {
          self.add_project();
          self.input = String::new();
          self.state = State::ProjectsList;
        }
        State::ProjectsInputUpdate => {
          self.update_project();
          self.input = String::new();
          self.state = State::ProjectsList;
        }
        _ => {}
      },
      KeyCode::Esc => {
        self.input = String::new();
        self.state = State::ProjectsList;
      }
      _ => {}
    }
  }

  pub fn add_project(&mut self) {
    let trimmed = self.input.trim();
    if trimmed.is_empty() {
      return;
    }
    match self.repo.add_project(trimmed) {
      Ok(_) => self.get_projects(),
      Err(_err) => {
        utils::notify("Error when creating a project");
      }
    };
  }

  pub fn update_project(&mut self) {
    if let Some(project) = self.get_highlighted_project() {
      match self.repo.update_project(project.id, &self.input) {
        Ok(_) => self.get_projects(),
        Err(_) => {
          utils::notify("Error when updating a project");
        }
      };
    }
  }

  pub fn get_projects(&mut self) {
    match self.repo.get_projects_in_progress() {
      Ok(projects) => self.projects_list.projects = projects,
      Err(err) => {
        println!("err: {:?}", err);
      }
    }
  }

  pub fn set_selected_project(&mut self, project_id: usize) {
    match self.projects_list.selected_id {
      None => match self.repo.set_selected(project_id, true) {
        Ok(()) => {
          self.projects_list.selected_id = Some(project_id);
        }
        Err(_) => {}
      },
      Some(id) => {
        let should_select = id != project_id;
        match self.repo.set_selected(project_id, should_select) {
          Ok(()) => {
            let new_selected_id = if should_select {
              Some(project_id)
            } else {
              None
            };
            self.projects_list.selected_id = new_selected_id
          }
          Err(_) => {}
        }
      }
    }
  }

  pub fn finish_project(&mut self) {
    match self.get_highlighted_project() {
      Some(project) => {
        match self.repo.mark_project_finished(project.id.clone()) {
          Ok(()) => {
            self.get_projects();
          }
          Err(_) => {}
        }
      }
      None => {}
    }
  }

  pub fn get_highlighted_project(&self) -> Option<&Project> {
    let highlighted_index = match self.projects_list.state.selected() {
      Some(index) => index,
      None => 0,
    };
    self.projects_list.projects.get(highlighted_index)
  }

  pub fn get_selected_project(&mut self) -> Option<&Project> {
    match self.projects_list.selected_id {
      None => None,
      Some(id) => self
        .projects_list
        .projects
        .iter()
        .find(|project| project.id == id),
    }
  }

  pub fn next_project(&mut self) {
    let i = match self.projects_list.state.selected() {
      None => 0,
      Some(index) => {
        if index >= self.projects_list.projects.len() - 1 {
          0
        } else {
          index + 1
        }
      }
    };
    self.projects_list.state.select(Some(i));
  }

  pub fn prev_project(&mut self) {
    let i = match self.projects_list.state.selected() {
      None => 0,
      Some(index) => {
        if index == 0 {
          self.projects_list.projects.len() - 1
        } else {
          index - 1
        }
      }
    };
    self.projects_list.state.select(Some(i));
  }

  pub fn list_projects(&mut self) {
    // TODO: refresh projects ?
    self.projects_list.state.select(Some(0));
    self.state = State::ProjectsList;
  }
}
