use std::time::SystemTime;

use super::{key_identifier::KeyIdentifier, key_state::KeyState};

pub struct KeyboardEvent {
    key: KeyIdentifier,
    value: KeyState,
    timestamp: SystemTime,
}
