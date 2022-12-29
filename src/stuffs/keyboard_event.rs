use std::{fmt::Display, time::SystemTime};

use super::{key_identifier::KeyIdentifier, key_state::KeyState};

#[derive(Getters, Debug, PartialEq, Eq)]
pub struct KeyboardEvent<'a> {
    #[getset(get = "pub")]
    key: KeyIdentifier<'a>,

    #[getset(get = "pub")]
    value: KeyState,

    #[getset(get = "pub")]
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

impl<'a> Display for KeyboardEvent<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.key, self.value)
    }
}

/// Test Keyboard Event
#[macro_export]
macro_rules! tke {
    ($device:ident $key:ident $value:ident $time:literal) => {{
        let key = $crate::stuffs::key_identifier::KeyIdentifier::new(&$device, stringify!($key));
        let event = KeyboardEvent::new(
            key,
            stringify!($value),
            $crate::test_utilities::mipoch($time),
        );
        event
    }};
    ($device:ident $key:ident $value:literal $time:literal) => {{
        let key = KeyIdentifier::new(&$device, stringify!($key));
        let event = KeyboardEvent::new(key, $value, mipoch($time));
        event
    }};
}

#[allow(non_snake_case)]
#[cfg(test)]
mod keyboard_event_module_test {
    use crate::{stuffs::keyboard::Keyboard, test_utilities::mipoch};

    use super::*;

    #[test]
    fn key_identifier_constructor_without_macro() {
        let L1 = Keyboard::new("L1", "My Left Keyboard", "usb/0/0/input0");
        let R1 = Keyboard::new("R1", "My Right Keyboard", "usb/1/1/input0");

        let L1_LEFTCTRL = KeyIdentifier::new(&L1, "LEFTCTRL");
        let _event_1 = KeyboardEvent::new(L1_LEFTCTRL, "Down", mipoch(0));

        let R1_J = KeyIdentifier::new(&R1, "J");
        let _event_2 = KeyboardEvent::new(R1_J, 1, mipoch(50));
    }

    #[test]
    fn key_identifier_constructor_with_tke_macro() {
        let L1 = Keyboard::new("L1", "My Left Keyboard", "usb/0/0/input0");
        let R1 = Keyboard::new("R1", "My Right Keyboard", "usb/1/1/input0");

        let _event_1 = tke!(L1 LEFTCTRL Down 0);
        let _event_2 = tke!(R1 J Down 50);
    }

    #[test]
    fn display_trait_implemented_for_keyboard_event_struct() {
        let L1 = Keyboard::new("L1", "My Left Keyboard", "usb/0/0/input0");
        let R1 = Keyboard::new("R1", "My Right Keyboard", "usb/1/1/input0");

        let event_1 = tke!(L1 LEFTCTRL Down 0);
        assert_eq!(event_1.to_string(), "L1 LEFTCTRL Down");

        let event_2 = tke!(R1 J Down 50);
        assert_eq!(event_2.to_string(), "R1 J Down");
    }
}
