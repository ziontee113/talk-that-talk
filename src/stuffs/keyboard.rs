#[derive(Getters)]
pub struct Keyboard {
    #[getset(get)]
    name: String,

    #[getset(get)]
    path: String,

    #[getset(get)]
    alias: String,
}

impl Keyboard {
    pub fn new<S: Into<String>>(name: S, path: S, alias: S) -> Self {
        Self {
            name: name.into(),
            path: path.into(),
            alias: alias.into(),
        }
    }
}

#[cfg(test)]
mod keyboard_module_test {
    use super::*;

    #[test]
    fn getset_getters_macro_test() {
        let keyboard = Keyboard::new("My Keyboard", "usb.0.1/input0", "L1");

        assert_eq!(keyboard.name(), "My Keyboard");
        assert_eq!(keyboard.path(), "usb.0.1/input0");
        assert_eq!(keyboard.alias(), "L1");
    }
}
