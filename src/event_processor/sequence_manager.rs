use crate::stuffs::{key_state::KeyState, keyboard_event::KeyboardEvent};

#[derive(Getters, Setters)]
pub struct SequenceManager<'a> {
    #[getset(get = "pub")]
    sequence: Vec<KeyboardEvent<'a>>,

    #[getset(get = "pub")]
    output: String,

    #[getset(get = "pub", set = "pub")]
    emitted: bool,
}

impl<'a> SequenceManager<'a> {
    pub fn new() -> Self {
        Self {
            sequence: vec![],
            output: String::new(),
            emitted: false,
        }
    }

    pub fn receive(&mut self, event: KeyboardEvent<'a>) {
        self.output.clear();

        match event.value() {
            KeyState::Down => {
                self.add_event(event);
                self.emitted = false;
            }
            KeyState::Up => {
                if event.key() == self.sequence.last().unwrap().key() {
                    self.output = self.sequence_as_string();
                }

                self.sequence.drain_filter(|e| e.key() == event.key());
            }
            KeyState::Hold => (),
        }
    }

    fn add_event(&mut self, event: KeyboardEvent<'a>) {
        self.sequence.push(event);
    }

    fn sequence_as_string(&self) -> String {
        self.sequence
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<String>>()
            .join(", ")
    }
}

#[allow(non_snake_case)]
#[cfg(test)]
mod sequence_manager_module_test {
    use crate::{stuffs::keyboard::Keyboard, tke};

    use super::*;

    #[test]
    fn can_print_sequence() {
        let L1 = Keyboard::new("L1", "My Left Keyboard", "usb/0/0/input0");
        let mut sm = SequenceManager::new();

        sm.receive(tke!(L1 LEFTCTRL Down 0));
        assert_eq!(sm.sequence_as_string(), "L1 LEFTCTRL Down");

        sm.receive(tke!(L1 J Down 50));
        assert_eq!(sm.sequence_as_string(), "L1 LEFTCTRL Down, L1 J Down");
    }

    #[test]
    fn pressing_J() {
        let L1 = Keyboard::new("L1", "My Left Keyboard", "usb/0/0/input0");
        let mut sm = SequenceManager::new();

        sm.receive(tke!(L1 J Down 0));
        assert_eq!(sm.output(), "");

        sm.receive(tke!(L1 J Up 100));
        assert_eq!(sm.output(), "L1 J Down");
    }

    #[test]
    fn CTRL_J_sequence() {
        let L1 = Keyboard::new("L1", "My Left Keyboard", "usb/0/0/input0");
        let mut sm = SequenceManager::new();

        sm.receive(tke!(L1 LEFTCTRL Down 0));
        assert_eq!(sm.output(), "");

        sm.receive(tke!(L1 J Down 100));
        assert_eq!(sm.output(), "");

        sm.receive(tke!(L1 J Up 200));
        assert_eq!(sm.output(), "L1 LEFTCTRL Down, L1 J Down");
    }
}
