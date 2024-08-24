use color_eyre::Result;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Alignment, Rect},
    style::Stylize,
    symbols::border,
    text::Line,
    widgets::{
        block::{Position, Title},
        Block, Paragraph, Widget,
    },
    Frame,
};
use std::time::{Duration, SystemTime};

mod errors;
mod tui;
mod utils;

pub struct Session {
    start: SystemTime,
    end: Option<SystemTime>,
    duration: i32,
}

impl Session {
    pub fn new() -> Self {
        Self {
            start: SystemTime::now(),
            end: None,
            duration: 25,
        }
    }
}

#[derive(Default)]
pub struct App {
    exit: bool,
    current_session: Option<Session>,
}

pub struct CounterWidget {
    time: String,
}

impl Widget for CounterWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let counter_widget = Block::bordered().title("Session running");
        Paragraph::new(format!("Time: {:?}", self.time).as_str())
            .centered()
            .block(counter_widget)
            .render(Rect::new(25, 25, 25, 10), buf);
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Title::from(" Tomato ".bold());
        let instructions = Title::from(Line::from(vec![
            " Start ".into(),
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
        if let Some(session) = &self.current_session {
            let counter_widget = CounterWidget {
                time: utils::render_timer(session.start, session.duration),
            };
            frame.render_widget(counter_widget, frame.area());
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
            _ => {}
        }
        // Ok(())
    }

    fn toggle_session(&mut self) {
        if self.current_session.is_none() {
            self.current_session = Some(Session::new());
        } else {
            self.current_session = None;
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
