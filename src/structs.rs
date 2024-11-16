use std::time::SystemTime;

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
