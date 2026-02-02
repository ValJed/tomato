pub mod calendar;
pub mod projects;
pub mod sessions;

use color_eyre;
use ratatui::{
  Frame,
  crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
  layout::{Alignment, Constraint::Length},
  style::{Color, Modifier, Style, Stylize, palette::tailwind::GRAY},
  text::Line,
  widgets::{
    Block, ListState, Padding, Paragraph,
    block::{Position, Title},
  },
};

use crate::repository::ProjectRepository;
use crate::structs::{
  App, CalendarSection, CalendarState, InputMode, Options, ProjectsList, State,
  UserConfig,
};
use crate::tui;
use crate::utils;
use crate::widgets::{
  CalendarWidget, ConfirmWidget, CounterWidget, InputWidget, ProjectsListWidget,
};
use std::time::Duration;

impl App {
  pub fn new(user_config: &UserConfig) -> App {
    let repo =
      ProjectRepository::new(&user_config).expect("DB instantiation failed");
    let projects = match repo.get_projects_in_progress() {
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
    let options = repo.create_of_get_options().unwrap_or(Options {
      id: 1,
      work_duration: 25,
      break_duration: 5,
      ask_before_work: false,
      ask_before_break: false,
    });

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
        sessions: vec![],
        list_state: ListState::default(),
        selected_section: CalendarSection::Calendar,
      },
      options,
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
      State::ConfirmFinished => frame.render_widget(
        ConfirmWidget {
          question: String::from(" Finish Project ? "),
        },
        frame.area(),
      ),

      State::WorkInput => frame.render_widget(
        InputWidget {
          title: " Set Time: ",
          width: 25,
          input: &self.input,
        },
        frame.area(),
      ),
      State::BreakInput => frame.render_widget(
        InputWidget {
          title: " Set Time: ",
          width: 25,
          input: &self.input,
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
          title: " Add Project ",
          width: 50,
          input: &self.input,
        },
        frame.area(),
      ),
      State::ProjectsInputUpdate => frame.render_widget(
        InputWidget {
          title: " Update Project ",
          width: 50,
          input: &self.input,
        },
        frame.area(),
      ),
      State::Calendar => frame.render_widget(
        CalendarWidget {
          selected_date: self.calendar.selected_date.unwrap(),
          sessions: &self.calendar.sessions,
          list_state: &mut self.calendar.list_state,
          selected_section: &self.calendar.selected_section,
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
        State::ConfirmFinished => {
          self.finish_project();
          self.state = State::ProjectsList;
        }
        _ => {}
      },
      KeyCode::Char('n') => match self.state {
        State::ConfirmBreak => self.start_work_input(),
        State::ConfirmWork => self.start_break_input(),
        State::ConfirmFinished => self.state = State::ProjectsList,
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
      KeyCode::Char(char) => {
        if "0123456789".contains(char) {
          self.input =
            String::from(self.input.to_string() + char.to_string().as_str())
        }
      }
      _ => {}
    }
  }

  fn exit(&mut self) {
    self.exit = true;
  }
}
