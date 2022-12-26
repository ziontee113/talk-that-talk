use std::time::SystemTime;

use super::{key_identifier::KeyIdentifier, key_state::KeyState};

pub struct KeyboardEvent<'a> {
    key: KeyIdentifier<'a>,
    value: KeyState,
    timestamp: SystemTime,
}
