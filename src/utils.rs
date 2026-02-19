use notify_rust::Notification;
use ratatui::{
  layout::{Constraint, Flex, Layout, Rect},
  text::Line,
};
use std::time::SystemTime;

pub fn render_timer(start: SystemTime, duration: u32) -> Option<u32> {
  let duration_secs = duration * 60;
  let time = SystemTime::now().duration_since(start).unwrap().as_secs() as u32;
  let countdown_secs = duration_secs - time;

  if countdown_secs < 1 {
    return None;
  }

  Some(countdown_secs)
}

pub fn render_timer_str(start: SystemTime, duration: u32) -> Option<String> {
  match render_timer(start, duration) {
    None => return None,
    Some(seconds) => Some(render_timer_seconds(seconds)),
  }
}

pub fn render_timer_seconds(seconds: u32) -> String {
  if seconds < 60 {
    return format!("{}s", seconds as i32).to_string();
  }
  let minutes = seconds / 60;
  let hours = minutes / 60;
  if hours != 0 {
    let remaining_minutes = minutes % 60;
    return format!("{}h {}m", hours, remaining_minutes);
  }
  let remaining_seconds = seconds % 60;

  return format!("{}m {}s", minutes, remaining_seconds);
}

pub fn get_spent_time(start: SystemTime, duration: u32) -> u32 {
  let duration_secs = duration * 60;
  match render_timer(start, duration) {
    None => duration_secs,
    Some(seconds) => duration_secs - seconds,
  }
}

pub fn center(
  area: Rect,
  horizontal: Constraint,
  vertical: Constraint,
) -> Rect {
  let [area] = Layout::horizontal([horizontal])
    .flex(Flex::Center)
    .areas(area);
  let [area] = Layout::vertical([vertical]).flex(Flex::Center).areas(area);
  area
}

pub fn notify(text: &str) {
  let notif = Notification::new()
    .summary("Tomato")
    .body(text)
    .appname("tomato")
    .show();

  match notif {
    Ok(_) => {}
    Err(_) => {}
  }
}

pub fn truncate(text: String, size: usize) -> String {
  if text.len() <= size {
    return text;
  }
  let mut cloned = text.clone();
  cloned.truncate(size);
  let formatted = cloned + "...";
  formatted
}

pub fn break_line(line: String, max_line_length: usize) -> String {
  if line.len() < max_line_length {
    return line;
  }
  let mut position = 0;
  let mut formatted = String::new();

  loop {
    let end = position + max_line_length;
    if end >= line.len() {
      let substring = &line[position..line.len()];
      formatted.push_str(substring);
      break;
    }

    let substring = &line[position..end];
    let space_pos = substring.rfind(' ').unwrap_or(substring.len());
    let space = if space_pos != substring.len() { 1 } else { 0 };
    let mut updated = substring.to_string();
    updated.replace_range(space_pos.., "\n");
    formatted.push_str(updated.as_str());
    position += space_pos + space;
  }

  formatted
}

pub fn convert_bool_to_string(value: bool) -> String {
  if value {
    return String::from("[X]");
  } else {
    return String::from("[ ]");
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn test_render_timer_seconds() {
    assert_eq!(render_timer_seconds(30), "30s");
    assert_eq!(render_timer_seconds(120), "2m 0s");
    assert_eq!(render_timer_seconds(125), "2m 5s");
    assert_eq!(render_timer_seconds(3600), "1h 0m");
    assert_eq!(render_timer_seconds(3800), "1h 3m");
  }

  #[test]
  fn test_render_timer() {
    let start = SystemTime::now();
    assert_eq!(render_timer_str(start, 1), Some("1m 0s".to_string()));
    assert_eq!(render_timer_str(start, 3), Some("3m 0s".to_string()));
  }
}
