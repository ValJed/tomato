use std::fmt::format;

use ratatui::{
  buffer::Buffer,
  layout::{
    Alignment,
    Constraint::{Fill, Length, Percentage},
    Rect,
  },
  prelude::{Direction, Layout},
  style::{Color, Modifier, Style, Stylize, palette::tailwind::GRAY},
  symbols::border,
  text::Line,
  widgets::{
    Block, List, ListItem, ListState, Padding, Paragraph, StatefulWidget,
    Widget,
    block::{Position, Title},
    calendar::{CalendarEventStore, Monthly},
  },
};
use time::Date;

use crate::structs::{
  App, CalendarSection, Project, SessionPerDay, SessionType, State,
};
use crate::utils::{
  break_line, center, convert_bool_to_string, render_timer_seconds, truncate,
};
use crate::{app::options::Options, utils::notify};

const SELECTED_STYLE: Style = Style::new().bg(Color::LightRed);

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
    let counter_area = center(area, Length(25), Length(5));

    let time = format!("Time: {}", self.time);
    Paragraph::new(time)
      .centered()
      .block(block)
      .render(counter_area, buf);
  }
}

pub struct ProjectsListWidget<'a> {
  pub projects: &'a [Project],
  pub selected_id: Option<usize>,
  pub state: &'a mut ListState,
}

impl Widget for ProjectsListWidget<'_> {
  fn render(self, area: Rect, buf: &mut Buffer) {
    let session_type = " Projects ";
    let title = Title::from(session_type.bold());
    let instructions = Title::from(Line::from(vec![
      " <A>".blue().bold(),
      " Add ".into(),
      "<F>".blue().bold(),
      " Finished ".into(),
      "<U>".blue().bold(),
      " Update ".into(),
      "<I>".blue().bold(),
      " Info ".into(),
    ]));
    let block = Block::bordered()
      .title(title.alignment(Alignment::Center))
      .title(
        instructions
          .alignment(Alignment::Center)
          .position(Position::Bottom),
      )
      .padding(Padding::new(1, 1, 1, 1));

    let highlighted_index = match self.state.selected() {
      Some(index) => index,
      None => 0,
    };

    let projects: Vec<ListItem> = self
      .projects
      .iter()
      .enumerate()
      .map(|(i, project)| {
        let is_selected = match self.selected_id {
          None => false,
          Some(id) => id == project.id,
        };

        let is_current = highlighted_index == i;
        let pre_content = if is_selected { "> " } else { "" };
        let content = pre_content.to_string() + &project.name.clone();
        if is_current {
          return ListItem::from(content).style(SELECTED_STYLE);
        }

        ListItem::from(content)
      })
      .collect();

    let list = List::new(projects).block(block);
    let list_area = center(area, Length(100), Length(10));
    StatefulWidget::render(list, list_area, buf, self.state);
  }
}

pub struct InputWidget<'a> {
  pub title: &'a str,
  pub width: u16,
  pub input: &'a str,
}

impl Widget for InputWidget<'_> {
  fn render(self, area: Rect, buf: &mut Buffer) {
    let title = Title::from(self.title);
    let instructions =
      Title::from(Line::from(vec![" <Enter>".blue().bold(), " Ok ".into()]));

    let block = Block::bordered()
      .title(title.alignment(Alignment::Left))
      .title(
        instructions
          .alignment(Alignment::Center)
          .position(Position::Bottom),
      )
      .padding(Padding::new(1, 1, 1, 1));
    let input_area = center(area, Length(self.width), Length(5));
    let available_width = input_area.width.saturating_sub(4) as usize; // 2 for borders, 2 for padding
    let scroll = self
      .input
      .len()
      .saturating_sub(available_width.saturating_sub(1));
    let paragraph = Paragraph::new(self.input)
      .scroll((0, scroll as u16))
      .block(block);

    paragraph.render(input_area, buf);
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
    let confirm_area = center(area, Length(25), Length(5));

    Paragraph::new("(y)es  (n)o")
      .centered()
      .block(block)
      .render(confirm_area, buf)
  }
}

pub struct CalendarWidget<'a> {
  pub selected_date: Date,
  pub sessions: &'a [SessionPerDay],
  pub selected_section: &'a CalendarSection,
  pub list_state: &'a mut ListState,
}

impl Widget for CalendarWidget<'_> {
  fn render(self, area: Rect, buf: &mut Buffer) {
    let layout_area = center(area, Length(50), Percentage(80));
    let layout = Layout::default()
      .direction(Direction::Vertical)
      .constraints(vec![Length(8), Fill(1)])
      .split(layout_area);
    let cal_layout = center(layout[0], Length(25), Percentage(100));
    let sessions_layout = layout[1];

    let sessions_title = Title::from(" Sessions ");
    let select_instruction = match self.selected_section {
      CalendarSection::Calendar => " Select List ",
      CalendarSection::List => " Select Calendar ",
    };
    let instructions = Title::from(Line::from(vec![
      " <Tab>".blue().bold(),
      select_instruction.into(),
    ]));
    let sessions_block = Block::bordered()
      .title(sessions_title.alignment(Alignment::Center))
      .title(
        instructions
          .alignment(Alignment::Center)
          .position(Position::Bottom),
      )
      .padding(Padding::new(1, 1, 1, 1));

    let cal_selected_color = match self.selected_section {
      CalendarSection::Calendar => Color::Red,
      CalendarSection::List => Color::Blue,
    };

    let mut cal_event = CalendarEventStore::default();
    cal_event.add(self.selected_date, Style::default().bg(cal_selected_color));
    let default_style = Style::default().add_modifier(Modifier::BOLD);
    let header_style = Style::default()
      .add_modifier(Modifier::BOLD)
      .add_modifier(Modifier::DIM)
      .fg(Color::Yellow);
    let cal = Monthly::new(
      Date::from_calendar_date(
        self.selected_date.year(),
        self.selected_date.month(),
        self.selected_date.day(),
      )
      .unwrap(),
      cal_event,
    )
    .show_weekdays_header(header_style)
    .default_style(default_style)
    .show_month_header(Style::default());

    let highlighted_index = match self.list_state.selected() {
      Some(index) => index,
      None => 0,
    };
    let sessions_list: Vec<ListItem> = self
      .sessions
      .iter()
      .enumerate()
      .map(|(i, session)| {
        let is_current = highlighted_index == i;
        let timer = render_timer_seconds(session.duration);
        let content = format!("{} - {}", session.project_name, timer);
        let wrapped = break_line(content, (sessions_layout.width - 4) as usize);

        if is_current && let CalendarSection::List = self.selected_section {
          return ListItem::from(wrapped).style(SELECTED_STYLE);
        }

        ListItem::from(wrapped)
      })
      .collect();

    let list = List::new(sessions_list).block(sessions_block);

    cal.render(cal_layout, buf);
    StatefulWidget::render(list, sessions_layout, buf, self.list_state);
  }
}

// Rendering the main application widget
impl Widget for &mut App {
  fn render(self, area: Rect, buf: &mut Buffer) {
    let title = Title::from(" 🍅 Tomato ".bold());
    let toggle_session = if self.current_session.is_none() {
      " Start "
    } else {
      " Stop "
    };

    let main_cmd = match self.state {
      State::WorkInput | State::BreakInput => "<Enter>",
      State::WorkDurationInput | State::BreakDurationInput => "<Enter>",
      _ => " <Space>",
    };
    let instructions = Title::from(Line::from(vec![
      main_cmd.blue().bold(),
      toggle_session.into(),
      "<P>".blue().bold(),
      " Projects ".into(),
      "<C>".blue().bold(),
      " Calendar ".into(),
      "<O>".blue().bold(),
      " Options ".into(),
      "<Q>".blue().bold(),
      " Quit ".into(),
    ]));
    let selected_project = self.get_selected_project();
    let selected_project_name = match selected_project {
      None => String::from("None"),
      Some(project) => project.name.clone(),
    };
    let project_title =
      Title::from(format!(" 📁 {} ", truncate(selected_project_name, 25)))
        .alignment(Alignment::Right)
        .position(Position::Top);
    Block::bordered()
      .title(title.alignment(Alignment::Left))
      .title(project_title)
      .title(
        instructions
          .alignment(Alignment::Center)
          .position(Position::Bottom),
      )
      .border_set(border::THICK)
      .render(area, buf);
  }
}

pub struct OptionsWidget<'a> {
  pub data: &'a Options,
  pub selected_index: usize,
}

impl Widget for OptionsWidget<'_> {
  fn render(self, area: Rect, buf: &mut Buffer) {
    let title = Title::from(" Options ".bold());
    let instructions = Title::from(Line::from(vec![
      " <Enter>".blue().bold(),
      " Update ".into(),
    ]));
    let options_area = center(area, Length(50), Length(8));

    let lines = self.data.get_list();
    let names_lines: Vec<Line> = lines
      .iter()
      .enumerate()
      .map(|(i, line)| {
        let text = if i == self.selected_index {
          format!("> {}", line.1)
        } else {
          line.1.clone()
        };

        Line::raw(text)
      })
      .collect();

    let values_lines: Vec<Line> =
      lines.iter().map(|line| Line::raw(&line.2)).collect();

    let block = Block::bordered()
      .title(title.alignment(Alignment::Left))
      // .padding(Padding::top(1))
      .padding(Padding::uniform(1))
      .title(
        instructions
          .alignment(Alignment::Center)
          .position(Position::Bottom),
      );
    let inner = block.inner(options_area);
    let layout = Layout::default()
      .direction(Direction::Horizontal)
      .constraints(vec![Fill(1), Length(5)])
      .split(inner);
    let names_layout = layout[0];
    let values_layout = layout[1];

    block.render(options_area, buf);

    Paragraph::new(names_lines).render(names_layout, buf);
    Paragraph::new(values_lines).render(values_layout, buf);
  }
}
