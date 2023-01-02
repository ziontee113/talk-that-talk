use std::fmt::Display;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum KeyState {
    Down,
    Up,
    Hold,
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
            _ => panic!("Invalid i32 KeyState Input"),
        }
    }
}

impl From<&str> for KeyState {
    fn from(input: &str) -> Self {
        match input.to_uppercase().as_str() {
            "DOWN" => KeyState::Down,
            "UP" => KeyState::Up,
            "HOLD" => KeyState::Hold,
            _ => panic!("Invalid &str KeyState Input"),
        }
    }
}
