use color_eyre::{owo_colors::OwoColorize, Result};
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Alignment, Constraint, Flex, Layout, Rect},
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{
        block::{Position, Title},
        Block, Padding, Paragraph, Widget,
    },
    Frame,
};
use std::time::{Duration, SystemTime};

mod errors;
mod structs;
mod tui;
mod utils;
mod widgets;

use structs::{Session, SessionType, State, UserConfig};
use widgets::{App, ConfirmWidget, CounterWidget, InputWidget};

const BREAK_DEFAULT: i32 = 5;
const WORK_DEFAULT: i32 = 25;

impl App {
    pub fn run(&mut self, terminal: &mut tui::Tui) -> Result<()> {
        let user_config: UserConfig =
            confy::load("tomato", "config").expect("Error when loading the config file");
        self.default_work_duration = user_config.default_work_duration;
        self.default_break_duration = user_config.default_break_duration;

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
                let time = utils::render_timer(session.start, session.duration);
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
                let time = utils::render_timer(session.start, session.duration);
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
            State::ConfirmWork => {
                frame.render_widget(
                    ConfirmWidget {
                        question: String::from(" Back to work? "),
                    },
                    frame.area(),
                );
            }
            State::WorkInput => frame.render_widget(
                InputWidget {
                    session_type: SessionType::Work,
                    input: self.input.clone(),
                },
                frame.area(),
            ),

            State::BreakInput => frame.render_widget(
                InputWidget {
                    session_type: SessionType::Break,
                    input: self.input.clone(),
                },
                frame.area(),
            ),
            _ => {}
        }
    }

    /// updates the application's state based on user input
    fn handle_events(&mut self, event: Event) {
        match event {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => match self.state {
                State::WorkInput => self.handle_input(key_event),
                State::BreakInput => self.handle_input(key_event),
                _ => self.handle_key_event(key_event),
            },
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
                _ => {}
            },
            KeyCode::Char('n') => match self.state {
                State::ConfirmBreak => self.start_work_input(),
                State::ConfirmWork => self.start_break_input(),
                _ => {}
            },
            KeyCode::Char('t') => match self.state {
                State::ConfirmBreak => {
                    self.input = self.default_break_duration.to_string();
                    self.state = State::BreakInput;
                }
                State::ConfirmWork => {
                    self.input = self.default_work_duration.to_string();
                    self.state = State::WorkInput
                }
                _ => {}
            },
            _ => {}
        }
        // Ok(())
    }

    fn handle_input(&mut self, key_event: KeyEvent) {
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

    fn start_work_input(&mut self) {
        self.input = WORK_DEFAULT.to_string();
        self.state = State::WorkInput
    }

    fn start_break_input(&mut self) {
        self.input = BREAK_DEFAULT.to_string();
        self.state = State::BreakInput;
    }

    fn start_work_session(&mut self) {
        let time: i32 = self.input.parse().unwrap_or(self.default_work_duration);
        self.state = State::WorkSession;
        self.current_session = Some(Session::new(SessionType::Work, time));
    }

    fn start_break_session(&mut self) {
        let time: i32 = self.input.parse().unwrap_or(self.default_break_duration);
        self.state = State::BreakSession;
        self.current_session = Some(Session::new(SessionType::Break, time));
    }

    fn toggle_session(&mut self) {
        if self.current_session.is_none() {
            self.start_work_session();
            return;
        }

        self.current_session = None;
        match self.state {
            State::WorkSession => {
                utils::notify("Break Time?");
                self.state = State::ConfirmBreak;
            }
            State::BreakSession => {
                self.state = State::ConfirmWork;
            }
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

fn main() -> Result<()> {
    errors::install_hooks()?;
    let mut terminal = tui::init()?;
    App::default().run(&mut terminal)?;
    tui::restore()?;
    Ok(())
}
