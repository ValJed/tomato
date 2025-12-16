use notify_rust::Notification;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
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
  let seconds_res = render_timer(start, duration);
  if seconds_res.is_none() {
    return None;
  }
  let seconds = seconds_res.unwrap();
  if seconds < 60 {
    return Some(format!("{}s", seconds as i32).to_string());
  }

  let minutes = seconds / 60;
  let remaining_seconds = minutes * 60;
  return Some(format!("{}m {}s", minutes, remaining_seconds));
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

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn test_to_time_str() {
    assert_eq!(to_time_str(30.0), "30s");
    assert_eq!(to_time_str(60.0), "1m 0s");
    assert_eq!(to_time_str(90.0), "1m 30s");
    assert_eq!(to_time_str(120.0), "2m 0s");
    assert_eq!(to_time_str(150.0), "2m 30s");
  }

  #[test]
  fn test_render_timer() {
    let start = SystemTime::now();
    assert_eq!(render_timer_str(start, 1), Some("1m 0s".to_string()));
    assert_eq!(render_timer_str(start, 3), Some("3m 0s".to_string()));
  }
}
