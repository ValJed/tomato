use color_eyre::{owo_colors::OwoColorize, Result};
use ratatui::{
    buffer::Buffer,
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
mod tui;
mod utils;

#[derive(Copy, Clone, Debug)]
enum SessionType {
    Work,
    Break,
}

// #[derive(Default)]
enum State {
    None,
    WorkSession,
    BreakSession,
    ConfirmBreak,
    ConfirmWork,
    ChooseTime,
}

impl Default for State {
    fn default() -> Self {
        State::None
    }
}

pub struct Session {
    start: SystemTime,
    end: Option<SystemTime>,
    duration: i32,
    session_type: SessionType,
}

impl Session {
    pub fn new(session_type: SessionType, duration: i32) -> Self {
        Self {
            start: SystemTime::now(),
            end: None,
            duration,
            session_type,
        }
    }
}

#[derive(Default)]
pub struct App {
    state: State,
    exit: bool,
    current_session: Option<Session>,
}

pub struct CounterWidget {
    time: String,
    session_type: SessionType,
}

impl Widget for CounterWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let session_type = if let SessionType::Work = self.session_type {
            " Work Session "
        } else {
            " Break Session "
        };
        let title = Title::from(session_type.bold());
        let block = Block::bordered()
            .title(title.alignment(Alignment::Center))
            .padding(Padding::new(1, 1, 1, 1));
        let counter_area = center(area, Constraint::Length(25), Constraint::Length(5));

        let time = format!("Time: {}", self.time);
        Paragraph::new(time)
            .centered()
            .block(block)
            .render(counter_area, buf);
    }
}

pub struct ConfirmWidget {
    question: String,
}

impl Widget for ConfirmWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Title::from(self.question);
        let block = Block::bordered()
            .title(title.alignment(Alignment::Center))
            .padding(Padding::new(1, 1, 1, 1));
        let confirm_area = center(area, Constraint::Length(25), Constraint::Length(5));

        Paragraph::new("(y)es  (n)o")
            .centered()
            .block(block)
            .render(confirm_area, buf)
    }
}

// Render the main application widget
impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Title::from(" Tomato ".bold());
        let toggle_session = if self.current_session.is_none() {
            " Start "
        } else {
            " Stop "
        };
        let instructions = Title::from(Line::from(vec![
            toggle_session.into(),
            "<Space>".blue().bold(),
            " History ".into(),
            "<H>".blue().bold(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ]));
        Block::bordered()
            .title(title.alignment(Alignment::Center))
            .title(
                instructions
                    .alignment(Alignment::Center)
                    .position(Position::Bottom),
            )
            .border_set(border::THICK)
            .render(area, buf);
    }
}

fn center(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    let [area] = Layout::horizontal([horizontal])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([vertical]).flex(Flex::Center).areas(area);
    area
}

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

    fn render_layout(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
        match &self.state {
            State::WorkSession => {
                let session = self.current_session.as_ref().unwrap();
                let counter_widget = CounterWidget {
                    time: utils::render_timer(session.start, session.duration),
                    session_type: session.session_type,
                };
                frame.render_widget(counter_widget, frame.area());
            }
            State::BreakSession => {
                let session = self.current_session.as_ref().unwrap();
                let counter_widget = CounterWidget {
                    time: utils::render_timer(session.start, session.duration),
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
        self.current_session = Some(Session::new(SessionType::Work, 25));
    }

    fn start_break_session(&mut self) {
        self.state = State::BreakSession;
        self.current_session = Some(Session::new(SessionType::Break, 5));
    }

    fn toggle_session(&mut self) {
        if self.current_session.is_none() {
            self.start_work_session();
            return;
        }

        self.current_session = None;
        match self.state {
            State::WorkSession => {
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
