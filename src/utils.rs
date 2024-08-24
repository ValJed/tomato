use std::time::SystemTime;

pub fn render_timer(start: SystemTime, duration: i32) -> String {
    let duration_secs: f32 = duration as f32 * 60.0;
    let time = SystemTime::now().duration_since(start).unwrap().as_secs() as f32;
    let countdown_secs = duration_secs - time;

    to_time_str(countdown_secs)
}

pub fn to_time_str(seconds: f32) -> String {
    if seconds < 60.0 {
        return format!("{}s", seconds as i32).to_string();
    }

    let minutes = seconds / 60.0;
    let minutes_whole = minutes.floor() as i32; // Get the whole number part of minutes
    let remaining_seconds = (minutes.fract() * 60.0).round() as i32; // Calculate remaining seconds

    return format!("{}m {}s", minutes_whole, remaining_seconds);
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
        assert_eq!(render_timer(start, 1), "1m 0s");
        assert_eq!(render_timer(start, 3), "3m 0s");
    }
}
