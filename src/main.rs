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

pub struct Session {
    start: SystemTime,
    end: Option<SystemTime>,
}

impl Session {
    pub fn new() -> Self {
        Self {
            start: SystemTime::now(),
            end: None,
        }
    }
}

#[derive(Default)]
pub struct App {
    exit: bool,
    current_session: Option<Session>,
}

pub struct CounterWidget {
    time: Duration,
}

impl Widget for CounterWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // println!("session running");
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
    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut tui::Tui) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| {
                self.render_layout(frame);
            })?;

            if event::poll(Duration::from_millis(250))? {
                let event = event::read()?;
                self.handle_events(event);
            }
        }
        Ok(())
    }

    fn render_layout(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.size());
        if let Some(session) = &self.current_session {
            let time = SystemTime::now().duration_since(session.start).unwrap();
            // let seconds = time.as_secs();
            let counter_widget = CounterWidget { time };
            frame.render_widget(counter_widget, frame.size());
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

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::Style;

    #[test]
    fn render() {
        let app = App::default();
        let mut buf = Buffer::empty(Rect::new(0, 0, 50, 4));

        app.render(buf.area, &mut buf);

        let mut expected = Buffer::with_lines(vec![
            "┏━━━━━━━━━━━━━━━━━━━━ Tomato ━━━━━━━━━━━━━━━━━━━━┓",
            "┃                    Value: 0                    ┃",
            "┃                                                ┃",
            "┗━ Decrement <Left> Increment <Right> Quit <Q> ━━┛",
        ]);
        let title_style = Style::new().bold();
        let counter_style = Style::new().yellow();
        let key_style = Style::new().blue().bold();
        expected.set_style(Rect::new(14, 0, 22, 1), title_style);
        expected.set_style(Rect::new(28, 1, 1, 1), counter_style);
        expected.set_style(Rect::new(13, 3, 6, 1), key_style);
        expected.set_style(Rect::new(30, 3, 7, 1), key_style);
        expected.set_style(Rect::new(43, 3, 4, 1), key_style);

        // note ratatui also has an assert_buffer_eq! macro that can be used to
        // compare buffers and display the differences in a more readable way
        assert_eq!(buf, expected);
    }

    #[test]
    fn handle_key_event() -> Result<()> {
        let mut app = App::default();
        app.handle_key_event(KeyCode::Right.into()).unwrap();
        assert_eq!(app.counter, 1);

        app.handle_key_event(KeyCode::Left.into()).unwrap();
        assert_eq!(app.counter, 0);

        let mut app = App::default();
        app.handle_key_event(KeyCode::Char('q').into()).unwrap();
        assert_eq!(app.exit, true);

        Ok(())
    }

    #[test]
    #[should_panic(expected = "attempt to subtract with overflow")]
    fn handle_key_event_panic() {
        let mut app = App::default();
        let _ = app.handle_key_event(KeyCode::Left.into());
    }

    #[test]
    fn handle_key_event_overflow() {
        let mut app = App::default();
        assert!(app.handle_key_event(KeyCode::Right.into()).is_ok());
        assert!(app.handle_key_event(KeyCode::Right.into()).is_ok());
        assert_eq!(
            app.handle_key_event(KeyCode::Right.into())
                .unwrap_err()
                .to_string(),
            "counter overflow"
        );
    }
}
