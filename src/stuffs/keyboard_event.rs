use std::time::SystemTime;

use super::{key_identifier::KeyIdentifier, key_state::KeyState};

pub struct KeyboardEvent<'a> {
    key: KeyIdentifier<'a>,
    value: KeyState,
    timestamp: SystemTime,
}

impl<'a> KeyboardEvent<'a> {
    pub fn new<T: Into<KeyState>>(key: KeyIdentifier<'a>, value: T, timestamp: SystemTime) -> Self {
        Self {
            key,
            value: value.into(),
            timestamp,
        }
    }
}

#[cfg(test)]
mod keyboard_event_module_test {
    use crate::{
        stuffs::{key_code::KeyCode, keyboard::Keyboard},
        test_utilities::mipoch,
    };

    use super::*;

    #[allow(non_snake_case, unused_variables)]
    #[test]
    fn key_identifier_constructor_test() {
        let L1 = Keyboard::new("L1", "My Left Keyboard", "usb/0/0/input0");
        let R1 = Keyboard::new("R1", "My Right Keyboard", "usb/1/1/input0");

        let L1_LEFTCTRL = KeyIdentifier::new(KeyCode::from("LEFTCTRL"), &L1);
        let event_1 = KeyboardEvent::new(L1_LEFTCTRL, 1, mipoch(0));

        let R1_J = KeyIdentifier::new(KeyCode::from("J"), &R1);
        let event_2 = KeyboardEvent::new(R1_J, 1, mipoch(50));
    }
}
