use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Rect},
    style::{palette::tailwind::GRAY, Color, Modifier, Style, Stylize},
    symbols::border,
    text::Line,
    widgets::{
        block::{Position, Title},
        Block, List, ListItem, ListState, Padding, Paragraph, Widget,
    },
};

use crate::structs::{App, Project, ProjectsList, SessionType, State};
use crate::utils::center;

pub struct InputWidget {
    pub input: String,
}

const SELECTED_STYLE: Style = Style::new().bg(GRAY.c400);
const UNSELECTED_STYLE: Style = Style::new().bg(GRAY.c400);

impl Widget for InputWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Title::from(" Set Time: ");
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

pub struct ProjectsListWidget<'a> {
    pub projects: &'a [Project],
    pub selected: Option<usize>,
    pub state: &'a mut ListState,
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

impl Widget for ProjectsListWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let session_type = " Projects ";
        let title = Title::from(session_type.bold());
        let block = Block::bordered()
            .title(title.alignment(Alignment::Center))
            .padding(Padding::new(1, 1, 1, 1));

        // let selected = match self.selected {
        //     Some(id) => Some(id),
        //     None => match self.projects {
        //         [] => None,
        //         projects => Some(projects[0].id),
        //     },
        // };
        let highlighted_index = match self.state.selected() {
            Some(index) => index,
            None => 0,
        };

        let projects: Vec<ListItem> = self
            .projects
            .iter()
            .enumerate()
            .map(|(i, project)| {
                // let is_selected = project.id == selected.unwrap();
                let is_current = highlighted_index == i;
                let content = project.name.clone();

                if is_current {
                    return ListItem::from(content).style(SELECTED_STYLE);
                }

                ListItem::from(content)
            })
            .collect();

        let list = List::new(projects).block(block);
        let list_area = center(area, Constraint::Length(100), Constraint::Length(20));
        Widget::render(list, list_area, buf);
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

// Render the main application widget
impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Title::from(" Tomato ".bold());
        let toggle_session = if self.current_session.is_none() {
            " Start "
        } else {
            " Stop "
        };
        let main_cmd = if let State::WorkInput = self.state {
            "<Enter>"
        } else if let State::BreakInput = self.state {
            "<Enter>"
        } else {
            "<Space>"
        };
        let projects = " Projects ";
        let instructions = Title::from(Line::from(vec![
            toggle_session.into(),
            main_cmd.blue().bold(),
            // Todo:
            projects.into(),
            "<P> ".blue().bold(),
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
