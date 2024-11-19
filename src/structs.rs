use serde::{Deserialize, Serialize};
use std::time::SystemTime;

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct UserConfig {
    pub default_work_duration: i32,
    pub default_break_duration: i32,
}

impl ::std::default::Default for UserConfig {
    fn default() -> Self {
        Self {
            default_work_duration: 25,
            default_break_duration: 5,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum SessionType {
    Work,
    Break,
}

// #[derive(Default)]
pub enum State {
    None,
    WorkSession,
    BreakSession,
    ConfirmBreak,
    ConfirmWork,
    ChooseTime,
    WorkInput,
    BreakInput,
}

impl Default for State {
    fn default() -> Self {
        State::None
    }
}

pub struct Session {
    pub start: SystemTime,
    pub end: Option<SystemTime>,
    pub duration: i32,
    pub session_type: SessionType,
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
