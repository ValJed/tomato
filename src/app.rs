pub mod calendar;
pub mod projects;
pub mod sessions;

use color_eyre;
use ratatui::{
  crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
  widgets::ListState,
  Frame,
};

use crate::repository::ProjectRepository;
use crate::structs::{App, CalendarState, ProjectsList, State, UserConfig};
use crate::tui;
use crate::utils;
use crate::widgets::{
  CalendarWidget, ConfirmWidget, CounterWidget, InputWidget, ProjectsListWidget,
};
use std::time::Duration;
use time::OffsetDateTime;

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
      repo,
      projects_list: ProjectsList {
        projects,
        selected_id,
        state: ListState::default(),
      },
      calendar: CalendarState {
        selected_date: None,
        list_state: ListState::default(),
      },
      default_work_duration: user_config.default_work_duration,
      default_break_duration: user_config.default_break_duration,
    }
  }

  pub fn run(&mut self, terminal: &mut tui::Tui) -> color_eyre::Result<()> {
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
          selected_date: self
            .calendar
            .selected_date
            .unwrap_or(OffsetDateTime::now_local().unwrap().date()),
          list_state: &mut self.calendar.list_state,
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

  fn exit(&mut self) {
    self.exit = true;
  }
}
