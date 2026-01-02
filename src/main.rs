use color_eyre::{owo_colors::OwoColorize, Result};
use ratatui::{
  crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
  layout::{Alignment, Constraint, Flex, Layout, Rect},
  style::Stylize,
  symbols::border,
  text::{Line, Text},
  widgets::{
    block::{Position, Title},
    Block, ListState, Padding, Paragraph, Widget,
  },
  Frame,
};
use std::time::{Duration, SystemTime};

mod errors;
mod repository;
mod structs;
mod tui;
mod utils;
mod widgets;

use repository::ProjectRepository;
use structs::{
  App, Project, ProjectsList, Session, SessionType, State, UserConfig,
};
use widgets::{
  CalendarWidget, ConfirmWidget, CounterWidget, InputWidget, ProjectsListWidget,
};

impl App {
  pub fn new(user_config: &UserConfig) -> App {
    let repo =
      ProjectRepository::new(&user_config).expect("DB instantiation failed");
    let projects = match repo.get_all_projects() {
      Ok(projs) => projs,
      Err(err) => {
        println!("err: {:?}", err);
        vec![]
      }
    };
    let selected_id = match projects.iter().find(|proj| proj.selected == true) {
      Some(proj) => Some(proj.id),
      None => None,
    };
    App {
      state: State::None,
      exit: false,
      current_session: None,
      input: String::new(),
      projects_list: ProjectsList {
        projects,
        selected_id,
        state: ListState::default(),
      },
      repo,
      default_work_duration: user_config.default_work_duration,
      default_break_duration: user_config.default_break_duration,
    }
  }

  pub fn run(&mut self, terminal: &mut tui::Tui) -> Result<()> {
    while !self.exit {
      terminal.draw(|frame| {
        self.render_layout(frame);
      })?;

      if event::poll(Duration::from_millis(100))? {
        let event = event::read()?;
        self.handle_events(event);
      }
    }
    Ok(())
  }

  fn render_layout(&mut self, frame: &mut Frame) {
    frame.render_widget(&mut *self, frame.area());
    match &self.state {
      State::WorkSession => {
        let session = self.current_session.as_ref().unwrap();
        let time = utils::render_timer_str(session.start, session.duration);
        if time.is_none() {
          self.toggle_session();
          return;
        }
        let counter_widget = CounterWidget {
          time: time.unwrap(),
          session_type: session.session_type,
        };
        frame.render_widget(counter_widget, frame.area());
      }
      State::BreakSession => {
        let session = self.current_session.as_ref().unwrap();
        let time = utils::render_timer_str(session.start, session.duration);
        if time.is_none() {
          self.toggle_session();
          return;
        }
        let counter_widget = CounterWidget {
          time: time.unwrap(),
          session_type: session.session_type,
        };
        frame.render_widget(counter_widget, frame.area());
      }
      State::ConfirmBreak => {
        frame.render_widget(
          ConfirmWidget {
            question: String::from(" Do you need a break? "),
          },
          frame.area(),
        );
      }
      State::ConfirmWork => frame.render_widget(
        ConfirmWidget {
          question: String::from(" Back to work? "),
        },
        frame.area(),
      ),
      State::ConfirmDelete => frame.render_widget(
        ConfirmWidget {
          question: String::from(" Delete Project ? "),
        },
        frame.area(),
      ),
      State::WorkInput => frame.render_widget(
        InputWidget {
          title: String::from(" Set Time: "),
          input: self.input.clone(),
          width: 25,
        },
        frame.area(),
      ),
      State::BreakInput => frame.render_widget(
        InputWidget {
          title: String::from(" Set Time: "),
          input: self.input.clone(),
          width: 25,
        },
        frame.area(),
      ),
      State::ProjectsList => frame.render_widget(
        ProjectsListWidget {
          projects: &self.projects_list.projects,
          selected_id: self.projects_list.selected_id,
          state: &mut self.projects_list.state,
        },
        frame.area(),
      ),
      State::ProjectsInputAdd => frame.render_widget(
        InputWidget {
          title: String::from(" Add Project "),
          input: self.input.clone(),
          width: 50,
        },
        frame.area(),
      ),
      State::ProjectsInputUpdate => frame.render_widget(
        InputWidget {
          title: String::from(" Update Project "),
          input: self.input.clone(),
          width: 50,
        },
        frame.area(),
      ),
      State::Calendar => frame.render_widget(
        CalendarWidget {
                // selected_date: ''
            },
        frame.area(),
      ),
      _ => {}
    }
  }

  // updates the application's state based on user input
  fn handle_events(&mut self, event: Event) {
    match event {
      Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
        match self.state {
          State::WorkInput | State::BreakInput => {
            self.handle_num_input(key_event)
          }
          State::ProjectsList => self.handle_projects_list_input(key_event),
          State::ProjectsInputAdd | State::ProjectsInputUpdate => {
            self.handle_project_input(key_event)
          }
          State::Calendar => self.handle_calendar_input(key_event),

          _ => self.handle_key_event(key_event),
        }
      }
      _ => {}
    }
  }

  fn handle_key_event(&mut self, key_event: KeyEvent) {
    match key_event.code {
      KeyCode::Char('q') => self.exit(),
      KeyCode::Char(' ') => self.toggle_session(),
      KeyCode::Char('y') => match self.state {
        State::ConfirmBreak => self.start_break_input(),
        State::ConfirmWork => self.start_work_input(),
        State::ConfirmDelete => {
          self.delete_project();
          self.state = State::ProjectsList;
        }
        _ => {}
      },
      KeyCode::Char('n') => match self.state {
        State::ConfirmBreak => self.start_work_input(),
        State::ConfirmWork => self.start_break_input(),
        State::ConfirmDelete => self.state = State::ProjectsList,
        _ => {}
      },
      // For now we can check projects only when not in a session
      KeyCode::Char('p') => match self.state {
        State::BreakSession => {}
        State::WorkSession => {}
        _ => {
          self.list_projects();
        }
      },
      KeyCode::Char('c') => match self.state {
        State::BreakSession => {}
        State::WorkSession => {}
        _ => {
          self.display_calendar();
        }
      },
      _ => {}
    }
  }

  fn handle_projects_list_input(&mut self, key_event: KeyEvent) {
    match key_event.code {
      KeyCode::Char('q') => self.exit(),
      KeyCode::Down | KeyCode::Char('j') => self.next_project(),
      KeyCode::Up | KeyCode::Char('k') => self.prev_project(),
      KeyCode::Char('p') => {
        self.state = State::None;
      }
      KeyCode::Char('a') => {
        self.state = State::ProjectsInputAdd;
      }
      KeyCode::Char('d') => {
        self.state = State::ConfirmDelete;
      }
      KeyCode::Char('u') => {
        self.input = match self.get_highlighted_project() {
          Some(project) => project.name.clone(),
          None => String::new(),
        };
        self.state = State::ProjectsInputUpdate;
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

  fn handle_project_input(&mut self, key_event: KeyEvent) {
    match key_event.code {
      KeyCode::Char(char) => {
        self.input = self.input.clone() + &char.to_string()
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
          // Create project
        }
        State::ProjectsInputUpdate => {
          self.update_project();
          self.input = String::new();
          self.state = State::ProjectsList;
        }
        _ => {}
      },
      _ => {}
    }
  }

  fn handle_calendar_input(&mut self, key_event: KeyEvent) {
    match key_event.code {
      KeyCode::Char('c') => self.state = State::None,
      _ => {}
    }
  }

  fn handle_num_input(&mut self, key_event: KeyEvent) {
    match key_event.code {
      KeyCode::Char('q') => self.exit(),
      KeyCode::Char('0') => self.input = self.input.clone() + "0",
      KeyCode::Char('1') => self.input = self.input.clone() + "1",
      KeyCode::Char('2') => self.input = self.input.clone() + "2",
      KeyCode::Char('3') => self.input = self.input.clone() + "3",
      KeyCode::Char('4') => self.input = self.input.clone() + "4",
      KeyCode::Char('5') => self.input = self.input.clone() + "5",
      KeyCode::Char('6') => self.input = self.input.clone() + "6",
      KeyCode::Char('7') => self.input = self.input.clone() + "7",
      KeyCode::Char('8') => self.input = self.input.clone() + "8",
      KeyCode::Char('9') => self.input = self.input.clone() + "9",
      KeyCode::Backspace => {
        let mut chars = self.input.chars();
        chars.next_back();
        self.input = chars.as_str().into();
      }
      KeyCode::Enter => match self.state {
        State::WorkInput => self.start_work_session(),
        State::BreakInput => self.start_break_session(),
        _ => {}
      },
      _ => {}
    }
  }

  fn add_project(&mut self) {
    match self.repo.add_project(self.input.clone()) {
      Ok(_) => self.get_projects(),
      Err(_err) => {
        utils::notify("Error when creating a project");
      }
    };
  }

  fn update_project(&mut self) {
    if let Some(project) = self.get_highlighted_project() {
      match self.repo.update_project(project.id, self.input.clone()) {
        Ok(_) => self.get_projects(),
        Err(_) => {
          utils::notify("Error when updating a project");
        }
      };
    }
  }

  fn get_projects(&mut self) {
    match self.repo.get_all_projects() {
      Ok(projects) => self.projects_list.projects = projects,
      Err(err) => {
        println!("err: {:?}", err);
      }
    }
  }

  fn set_selected_project(&mut self, project_id: usize) {
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

  fn delete_project(&mut self) {
    match self.get_highlighted_project() {
      Some(project) => match self.repo.delete_project(project.id.clone()) {
        Ok(()) => {
          self.get_projects();
        }
        Err(_) => {}
      },
      None => {}
    }
  }

  fn get_highlighted_project(&self) -> Option<&Project> {
    let highlighted_index = match self.projects_list.state.selected() {
      Some(index) => index,
      None => 0,
    };
    self.projects_list.projects.get(highlighted_index)
  }

  // Global methods
  fn get_selected_project(&mut self) -> Option<&Project> {
    match self.projects_list.selected_id {
      None => None,
      Some(id) => self
        .projects_list
        .projects
        .iter()
        .find(|project| project.id == id),
    }
  }

  fn next_project(&mut self) {
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

  fn prev_project(&mut self) {
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

  fn start_work_input(&mut self) {
    self.input = self.default_work_duration.to_string();
    self.state = State::WorkInput
  }

  fn list_projects(&mut self) {
    // TODO: refresh projects ?
    self.projects_list.state.select(Some(0));
    self.state = State::ProjectsList;
  }

  fn display_calendar(&mut self) {
    self.state = State::Calendar;
  }

  fn start_break_input(&mut self) {
    self.input = self.default_break_duration.to_string();
    self.state = State::BreakInput;
  }

  fn start_work_session(&mut self) {
    let time: u32 = self.input.parse().unwrap_or(self.default_work_duration);
    self.state = State::WorkSession;
    self.current_session = Some(Session::new(SessionType::Work, time));
  }

  fn start_break_session(&mut self) {
    let time: u32 = self.input.parse().unwrap_or(self.default_break_duration);
    self.state = State::BreakSession;
    self.current_session = Some(Session::new(SessionType::Break, time));
  }

  fn stop_work_session(&mut self) {
    let session = self.current_session.as_ref().unwrap();
    let spent_time = utils::get_spent_time(session.start, session.duration);

    if let Some(project_id) = self.get_selected_project().map(|p| p.id.clone())
    {
      let updated = self.repo.add_session(project_id.clone(), spent_time);
      if updated.is_err() {
        let err = updated.unwrap();

        println!("err: {:?}", err);
        utils::notify("Error when updating project spent time");
      }
    }
  }

  fn toggle_session(&mut self) {
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

  fn exit(&mut self) {
    self.exit = true;
  }
}

fn main() -> Result<()> {
  errors::install_hooks()?;
  let mut terminal = tui::init()?;
  let user_config: UserConfig = confy::load("tomato", "config")
    .expect("Error when loading the config file");
  App::new(&user_config).run(&mut terminal)?;
  tui::restore()?;
  Ok(())
}
