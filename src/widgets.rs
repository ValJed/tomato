use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Rect},
    style::Stylize,
    symbols::border,
    text::Line,
    widgets::{
        block::{Position, Title},
        Block, Padding, Paragraph, Widget,
    },
};

use crate::structs::{Session, SessionType, State};
use crate::utils::center;

pub struct InputWidget {
    pub input: String,
    pub session_type: SessionType,
}

impl Widget for InputWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let widget_title = if let SessionType::Work = self.session_type {
            " Work Session Time: "
        } else {
            " Break Session Time: "
        };
        let title = Title::from(widget_title);
        let block = Block::bordered()
            .title(title.alignment(Alignment::Left))
            .padding(Padding::new(1, 1, 1, 1));
        let counter_area = center(area, Constraint::Length(25), Constraint::Length(5));

        Paragraph::new(self.input)
            .centered()
            .block(block)
            .render(counter_area, buf);
    }
}

pub struct CounterWidget {
    pub time: String,
    pub session_type: SessionType,
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
    pub question: String,
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

#[derive(Default)]
pub struct App {
    pub state: State,
    pub exit: bool,
    pub current_session: Option<Session>,
    pub input: String,
}

// Render the main application widget
impl Widget for &mut App {
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
