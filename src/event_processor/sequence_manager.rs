use crate::stuffs::{key_state::KeyState, keyboard_event::KeyboardEvent};

#[derive(Getters)]
struct SequenceManager<'a> {
    #[getset(get = "pub")]
    sequence: Vec<KeyboardEvent<'a>>,
}

impl<'a> SequenceManager<'a> {
    fn new() -> Self {
        Self { sequence: vec![] }
    }

    fn receive(&mut self, event: KeyboardEvent<'a>) {
        match event.value() {
            KeyState::Down => {
                self.add_event(event);
            }
            KeyState::Up => {
                if event.key() == self.sequence.last().unwrap().key() {
                    self.sequence_as_string();

                    self.sequence.pop();
                } else {
                    self.add_event(event);
                }
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
    fn can_add_down_event_to_sequence() {
        let L1 = Keyboard::new("L1", "My Left Keyboard", "usb/0/0/input0");
        let R1 = Keyboard::new("R1", "My Right Keyboard", "usb/1/1/input0");
        let mut sm = SequenceManager::new();

        sm.receive(tke!(L1 LEFTCTRL Down 0));
        assert_eq!(*sm.sequence().get(0).unwrap(), tke!(L1 LEFTCTRL Down 0));

        sm.receive(tke!(R1 J Down 50));
        assert_eq!(*sm.sequence().get(1).unwrap(), tke!(R1 J Down 50));
    }

    #[test]
    fn can_remove_sequence_element_on_key_up() {
        let L1 = Keyboard::new("L1", "My Left Keyboard", "usb/0/0/input0");
        let mut sm = SequenceManager::new();

        sm.receive(tke!(L1 LEFTCTRL Down 0));
        sm.receive(tke!(L1 J Down 50));
        sm.receive(tke!(L1 J Up 100));
        assert_eq!(sm.sequence().len(), 1);
    }

    #[test]
    fn can_add_key_up_event_to_sequence_if_didnt_match_last_element_key() {
        let L1 = Keyboard::new("L1", "My Left Keyboard", "usb/0/0/input0");
        let mut sm = SequenceManager::new();

        sm.receive(tke!(L1 LEFTCTRL Down 0));
        sm.receive(tke!(L1 J Down 100));
        sm.receive(tke!(L1 LEFTCTRL Up 200));

        assert_eq!(sm.sequence().len(), 3);

        assert_eq!(*sm.sequence().get(0).unwrap(), tke!(L1 LEFTCTRL Down 0));
        assert_eq!(*sm.sequence().get(1).unwrap(), tke!(L1 J Down 100));
        assert_eq!(*sm.sequence().get(2).unwrap(), tke!(L1 LEFTCTRL Up 200));
    }

    #[test]
    fn can_print_sequence() {
        let L1 = Keyboard::new("L1", "My Left Keyboard", "usb/0/0/input0");
        let mut sm = SequenceManager::new();

        sm.receive(tke!(L1 LEFTCTRL Down 0));
        assert_eq!(sm.sequence_as_string(), "L1 LEFTCTRL Down");

        sm.receive(tke!(L1 J Down 50));
        assert_eq!(sm.sequence_as_string(), "L1 LEFTCTRL Down, L1 J Down");
    }
}
