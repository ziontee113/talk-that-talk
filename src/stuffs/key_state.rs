use std::fmt::Display;

#[derive(Debug)]
pub enum KeyState {
    Down,
    Up,
    Hold,
    Uninitiated,
}

impl Display for KeyState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl From<i32> for KeyState {
    fn from(value: i32) -> Self {
        match value {
            0 => KeyState::Up,
            1 => KeyState::Down,
            2 => KeyState::Hold,
            -1 => KeyState::Uninitiated,
            _ => unreachable!(),
        }
    }
}
