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
}

impl<'a> SequenceManager<'a> {
    pub fn new() -> Self {
        Self {
            sequence: vec![],
            output: String::new(),
            emitted: false,
            modifiers: vec![],
        }
    }

    pub fn receive(&mut self, event: KeyboardEvent<'a>) {
        self.output.clear();

        self.update_modifiers(&event);

        match event.value() {
            KeyState::Down => {
                self.add_event(event);
                self.emitted = false;
            }
            KeyState::Up => {
                let last_sequence_key = self.sequence.last().unwrap().key();
                if last_sequence_key == event.key() {
                    self.output.push_str(&self.produce_output());
                }

                self.sequence.drain_filter(|e| e.key() == event.key());
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
        if self.sequence.len() >= 2 {
            self.produce_union_output();
        }

        self.produce_non_union_result()
    }

    fn produce_union_output(&self) -> Vec<usize> {
        let interval_limit = 30;
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

        breakpoints
    }

    fn produce_non_union_result(&self) -> String {
        self.sequence
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<String>>()
            .join(", ")
    }
}

enum SequenceElement<'a> {
    Union(Union<'a>),
    Event(KeyboardEvent<'a>),
}

struct Union<'a>(Vec<KeyboardEvent<'a>>);

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

    #[test]
    fn produce_output_test() {
        let L1 = Keyboard::new("L1", "My Left Keyboard", "usb/0/0/input0");
        let mut sm = SequenceManager::new();

        sm.receive(tke!(L1 D Down 0));
        sm.receive(tke!(L1 F Down 20));

        let want: Vec<usize> = vec![0];
        let got = sm.produce_union_output();
        assert_eq!(want, got);
    }

    #[test]
    fn produce_output_test_2() {
        let L1 = Keyboard::new("L1", "My Left Keyboard", "usb/0/0/input0");
        let mut sm = SequenceManager::new();

        sm.receive(tke!(L1 D Down 0));
        sm.receive(tke!(L1 F Down 20));
        sm.receive(tke!(L1 J Down 100));

        let want: Vec<usize> = vec![0, 2];
        let got = sm.produce_union_output();
        assert_eq!(want, got);
    }

    #[test]
    fn produce_output_test_3() {
        let L1 = Keyboard::new("L1", "My Left Keyboard", "usb/0/0/input0");
        let mut sm = SequenceManager::new();

        sm.receive(tke!(L1 D Down 0));
        sm.receive(tke!(L1 F Down 20));
        sm.receive(tke!(L1 J Down 100));
        sm.receive(tke!(L1 K Down 120));
        sm.receive(tke!(L1 SPACE Down 200));
        sm.receive(tke!(L1 RIGHTALT Down 250));

        let want: Vec<usize> = vec![0, 2, 4, 5];
        let got = sm.produce_union_output();
        assert_eq!(want, got);
    }

    // #[test]
    // fn range_test() {
    //     let vec = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];
    //     let breakpoints: Vec<usize> = vec![0, 2, 4, 5];
    //
    //     // let mut ranges = vec![];
    //
    //     // for (i, point) in breakpoints.iter().enumerate() {
    //     //     if i + 1 < breakpoints.len() {
    //     //         let range = [point..breakpoints.get(i + 1).unwrap()];
    //     //         ranges.push(range);
    //     //     }
    //     //
    //     //     if i + 1 == breakpoints.len() {
    //     //         let point_later = point + 1;
    //     //         let range = [point..point + 1];
    //     //         ranges.push(range);
    //     //     }
    //     // }
    //
    //     assert_eq!(vec[0..2], vec![1, 2]);
    //     assert_eq!(vec[2..4], vec![3, 4]);
    //     assert_eq!(vec[4..5], vec![5]);
    //     assert_eq!(vec[5..=5], vec![6]);
    // }
}
