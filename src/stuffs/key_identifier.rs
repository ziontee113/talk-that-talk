use std::fmt::Display;

use super::{key_code::KeyCode, keyboard::Keyboard};

#[derive(Debug, PartialEq, Eq, Getters, PartialOrd, Ord, Clone)]
pub struct KeyIdentifier<'a> {
    device: &'a Keyboard,

    #[getset(get = "pub")]
    code: KeyCode,
}

impl<'a> Display for KeyIdentifier<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.device.alias(), self.code)
    }
}

impl<'a> KeyIdentifier<'a> {
    pub fn new<T: Into<KeyCode>>(device: &'a Keyboard, code: T) -> Self {
        Self {
            code: code.into(),
            device,
        }
    }
}

#[cfg(test)]
mod key_identifer_module_test {
    use super::KeyIdentifier;
    use crate::stuffs::keyboard::Keyboard;

    #[test]
    fn display_trait_for_key_identifier_implemented() {
        let device = Keyboard::new("L1", "My Keyboard", "usb/0/0/input0");

        let key_identifier = KeyIdentifier::new(&device, 32);
        assert_eq!(key_identifier.to_string(), "L1 D");
    }
}
