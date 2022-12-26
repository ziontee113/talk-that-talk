#[derive(Getters)]
pub struct Keyboard {
    #[getset(get = "pub")]
    alias: String,

    #[getset(get = "pub")]
    name: String,

    #[getset(get = "pub")]
    path: String,
}

impl Keyboard {
    pub fn new<S: Into<String>>(alias: S, name: S, path: S) -> Self {
        Self {
            alias: alias.into(),
            name: name.into(),
            path: path.into(),
        }
    }
}

#[cfg(test)]
mod keyboard_module_test {
    use super::*;

    #[test]
    fn getset_getters_macro_test() {
        let keyboard = Keyboard::new("L1", "My Keyboard", "usb.0.1/input0");

        assert_eq!(keyboard.alias(), "L1");
        assert_eq!(keyboard.name(), "My Keyboard");
        assert_eq!(keyboard.path(), "usb.0.1/input0");
    }
}
