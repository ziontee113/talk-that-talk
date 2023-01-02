use std::fmt::Display;

use crate::stuffs::{key_state::KeyState, keyboard_event::KeyboardEvent};

#[derive(Getters, Setters)]
pub struct SequenceManager<'a> {
    #[getset(get = "pub")]
    sequence: Vec<KeyboardEvent<'a>>,

    #[getset(get = "pub")]
    output: String,

    #[getset(get = "pub", set = "pub")]
    emitted: bool,

    #[getset(get = "pub")]
    modifiers: Vec<u16>,

    currently_down_events: Vec<KeyboardEvent<'a>>,
}

impl<'a> SequenceManager<'a> {
    pub fn new() -> Self {
        Self {
            sequence: vec![],
            output: String::new(),
            emitted: false,
            modifiers: vec![],
            currently_down_events: vec![],
        }
    }

    pub fn receive(&mut self, event: KeyboardEvent<'a>) {
        self.output.clear();

        self.update_modifiers(&event);

        match event.value() {
            KeyState::Down => {
                self.add_event(event.clone());
                self.emitted = false;

                self.currently_down_events.push(event);
            }
            KeyState::Up => {
                let last_down_key = self.currently_down_events.last().unwrap().key().clone();

                if event.key() != &last_down_key {
                    let down_event = self
                        .currently_down_events
                        .iter()
                        .find(|e| e.key() == event.key() && *e.value() == KeyState::Down)
                        .unwrap();

                    if event
                        .timestamp()
                        .duration_since(*down_event.timestamp())
                        .unwrap()
                        .as_millis()
                        > 100
                    {
                        self.add_event(event.clone());
                    } else {
                        self.sequence.drain_filter(|e| e.key() == event.key());
                    }
                }

                // ---------------------------------------------

                let last_sequence_key = self.sequence.last().unwrap().key();
                if event.key() == last_sequence_key {
                    self.output.push_str(&self.produce_output());
                }

                // ---------------------------------------------

                self.currently_down_events
                    .drain_filter(|e| e.key() == event.key());

                if event.key() == &last_down_key {
                    self.sequence.drain_filter(|e| e.key() == event.key());
                }

                // ---------------------------------------------

                if self.currently_down_events.is_empty() {
                    self.sequence.clear();
                }
            }
            KeyState::Hold => (),
        }
    }

    pub fn first_code(&self) -> u16 {
        self.sequence.first().unwrap().key().code().0
    }

    pub fn is_combined(&self) -> bool {
        self.output.contains(',')
    }

    fn update_modifiers(&mut self, event: &KeyboardEvent) {
        let modifier_codes: Vec<u16> = vec![14, 29, 42, 54, 56, 97, 100, 125, 126];

        if self.emitted || self.sequence.is_empty() {
            self.modifiers.clear();
        } else if modifier_codes.contains(&event.key().code().0) {
            self.modifiers.push(event.key().code().0);
        }
    }

    fn add_event(&mut self, event: KeyboardEvent<'a>) {
        self.sequence.push(event);
    }

    fn produce_output(&self) -> String {
        {
            let interval_limit = 50;
            let mut breakpoints: Vec<usize> = vec![0];
            let mut last_time = self.sequence.first().unwrap().timestamp();

            for (i, e) in self.sequence.iter().enumerate() {
                if e.timestamp()
                    .duration_since(*last_time)
                    .unwrap()
                    .as_millis()
                    > interval_limit
                {
                    breakpoints.push(i);
                    last_time = e.timestamp();
                }
            }

            // ---------------------------------- refactor me please

            let mut unions = vec![];

            for (i, e) in self.sequence.iter().enumerate() {
                if breakpoints.contains(&i) {
                    unions.push(Union(vec![e]));
                } else {
                    unions.last_mut().unwrap().0.push(e);
                }
            }

            unions
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<String>>()
                .join(", ")
        }
    }
}

enum SequenceElement<'a> {
    Union(Union<'a>),
    Event(KeyboardEvent<'a>),
}

struct Union<'a>(Vec<&'a KeyboardEvent<'a>>);

impl<'a> Display for Union<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.len() == 1 {
            return write!(f, "{}", self.0.first().unwrap());
        }

        let mut sorted_union = self.0.clone();
        sorted_union.sort_by_key(|e| e.key());

        write!(
            f,
            "[{}]",
            sorted_union
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<String>>()
                .join(", ")
        )
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
        assert_eq!(sm.produce_output(), "L1 LEFTCTRL Down");

        sm.receive(tke!(L1 J Down 50));
        assert_eq!(sm.produce_output(), "L1 LEFTCTRL Down, L1 J Down");
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
