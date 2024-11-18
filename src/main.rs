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

use structs::{Session, SessionType, State};
use widgets::{App, ConfirmWidget, CounterWidget};

impl App {
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
            _ => {}
        }
    }

    /// updates the application's state based on user input
    fn handle_events(&mut self, event: Event) {
        match event {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event);
            }
            _ => {}
        }
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char(' ') => self.toggle_session(),
            KeyCode::Char('y') => match self.state {
                State::ConfirmBreak => self.start_break_session(),
                State::ConfirmWork => self.start_work_session(),
                _ => {}
            },
            KeyCode::Char('n') => match self.state {
                State::ConfirmBreak => self.start_work_session(),
                State::ConfirmWork => self.start_break_session(),
                _ => {}
            },
            _ => {}
        }
        // Ok(())
    }

    fn start_work_session(&mut self) {
        self.state = State::WorkSession;
        self.current_session = Some(Session::new(SessionType::Work, 1));
    }

    fn start_break_session(&mut self) {
        self.state = State::BreakSession;
        self.current_session = Some(Session::new(SessionType::Break, 1));
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
