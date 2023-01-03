use std::fmt::Display;

use crate::stuffs::{key_state::KeyState, keyboard_event::KeyboardEvent};

#[derive(Default)]
struct Sequence<'a> {
    elements: Vec<KeyboardEvent<'a>>,
}

impl<'a> Sequence<'a> {
    fn add(&mut self, event: KeyboardEvent<'a>) {
        self.elements.push(event);
    }
}

impl<'a> Display for Sequence<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.elements
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

#[derive(Default)]
pub struct NeoSequenceManager<'a> {
    sequence: Sequence<'a>,
}

impl<'a> NeoSequenceManager<'a> {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn receive(&mut self, event: KeyboardEvent<'a>) {
        match event.value() {
            KeyState::Down => {
                self.sequence.add(event);
            }
            KeyState::Up => {
                todo!()
            }
            KeyState::Hold => {
                todo!()
            }
        }
    }

    pub fn output(&self) -> String {
        self.sequence.to_string()
    }
}

#[allow(non_snake_case, unused_variables)]
#[cfg(test)]
mod neo_sequence_manager_module_test {
    use crate::{stuffs::keyboard::Keyboard, tke};

    use super::*;

    fn mock_keyboards() -> (Keyboard, Keyboard) {
        let L1 = Keyboard::new("L1", "My Left Keyboard", "usb/0/0/input0");
        let R1 = Keyboard::new("R1", "My Right Keyboard", "usb/1/1/input0");
        (L1, R1)
    }

    #[test]
    fn can_receive_event() {
        let (L1, R1) = mock_keyboards();
        let mut sm = NeoSequenceManager::new();

        sm.receive(tke!(R1 J Down 0));
        assert_eq!(sm.output(), "R1 J");
    }
}
