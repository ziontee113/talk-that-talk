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
                self.sequence.push(event);
            }
            KeyState::Up => {
                if event.key() == self.sequence.last().unwrap().key() {
                    self.sequence.pop();
                }
            }
            KeyState::Hold => (),
        }
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

        let mut manager = SequenceManager::new();

        let event = tke!(L1 LEFTCTRL Down 0);
        manager.receive(event);
        assert_eq!(
            *manager.sequence().get(0).unwrap(),
            tke!(L1 LEFTCTRL Down 0)
        );

        let event = tke!(R1 J Down 50);
        manager.receive(event);
        assert_eq!(*manager.sequence().get(1).unwrap(), tke!(R1 J Down 50));
    }

    #[test]
    fn can_remove_sequence_element_on_key_up() {
        let L1 = Keyboard::new("L1", "My Left Keyboard", "usb/0/0/input0");
        let mut manager = SequenceManager::new();

        manager.receive(tke!(L1 LEFTCTRL Down 0));
        manager.receive(tke!(L1 J Down 50));
        manager.receive(tke!(L1 J Up 100));
        assert_eq!(manager.sequence().len(), 1);
    }
}
