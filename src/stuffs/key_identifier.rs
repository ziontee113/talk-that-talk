use super::{key_code::KeyCode, keyboard::Keyboard};

pub struct KeyIdentifier {
    code: KeyCode,
    device: Keyboard,
}

impl KeyIdentifier {
    pub fn new<T: Into<KeyCode>>(code: T, device: Keyboard) -> Self {
        Self {
            code: code.into(),
            device,
        }
    }
}

#[cfg(test)]
mod key_identifer_module_test {
    use crate::stuffs::key_code::KeyCode;

    #[test]
    fn into_and_from_keycode_test() {
        let keycode = KeyCode::from(1);
        assert_eq!(keycode.0, 1);

        // let keycode = KeyCode::from("esc");
        // assert_eq!(keycode.0, 1);
    }

    // #[test]
    // fn display_trait_for_key_identifier_implemented() {
    //     let device = Keyboard::new("My Keyboard", "usb/0/0/input0", "L1");
    //
    //     let key_identifier = KeyIdentifier::new(32, device);
    // }
}
