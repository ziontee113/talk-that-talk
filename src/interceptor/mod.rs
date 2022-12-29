use std::{
    collections::HashMap,
    sync::mpsc::{self, Sender},
    thread,
    time::SystemTime,
};

use crate::{
    devices::{self, input::EventKindCheck, output::event_from_code},
    event_processor::sequence_manager::SequenceManager,
    stuffs::{key_identifier::KeyIdentifier, keyboard::Keyboard, keyboard_event::KeyboardEvent},
};

enum TransmitSignal {
    Key(String, u16, i32, SystemTime),
}

// for development purposes only
fn mock_device_alias() -> HashMap<&'static str, &'static str> {
    HashMap::from([
        ("L1", "usb-0000:00:1d.0-1.5.1.4/input0"),
        ("R1", "usb-0000:00:1d.0-1.5.2/input0"),
    ])
}

fn rules_to_print() -> HashMap<&'static str, &'static str> {
    HashMap::from([
        ("L1 LEFTCTRL Down, R1 J Down", "my first mapping"),
        ("L1 LEFTCTRL Down, R1 K Down", "my second mapping"),
        ("L1 S Down, R1 O Down", "my third mapping: SO"),
        ("R1 O Down, L1 S Down", "my fourth mapping: O then S"),
        ("L1 CAPSLOCK Down", "MAP_CODE: 1"),
        ("L1 CAPSLOCK Down, R1 H Down", "MAP_CODE: 105"),
        ("L1 CAPSLOCK Down, R1 J Down", "MAP_CODE: 108"),
        ("L1 CAPSLOCK Down, R1 K Down", "MAP_CODE: 103"),
        ("L1 CAPSLOCK Down, R1 L Down", "MAP_CODE: 106"),
    ])
}

pub fn start() {
    // Development Variables
    let alias_map = mock_device_alias();
    let rules_to_print = rules_to_print();

    // Message Channels
    let (tx, rx) = mpsc::channel();

    for (device_alias, device_path) in alias_map {
        intercept(tx.clone(), device_alias, device_path);
    }

    // Interception
    let mut virtual_device = devices::output::new().unwrap();
    let mut sm = SequenceManager::new();

    let l1 = Keyboard::new("L1", "My Left Keyboard", "usb/0/0/input0");
    let r1 = Keyboard::new("R1", "My Right Keyboard", "usb/1/1/input0");

    for signal in rx {
        match signal {
            TransmitSignal::Key(device_alias, code, value, timestamp) => {
                // HACK:
                let mut device = &l1;
                if device_alias != "L1" {
                    device = &r1;
                }

                let key = KeyIdentifier::new(device, code);
                let event = KeyboardEvent::new(key, value, timestamp);

                sm.receive(event);

                // FRAUD:
                if let Some(msg) = rules_to_print.get(sm.output().as_str()) {
                    let pat = "MAP_CODE: ";
                    let split: Vec<&str> = msg.split("MAP_CODE: ").collect();

                    if msg.contains(pat) {
                        let code: u16 = split.last().unwrap().parse().unwrap();

                        let kb_down_event = event_from_code(code, 1);
                        let kb_up_event = event_from_code(code, 0);

                        if !sm.emitted() {
                            virtual_device.emit(&[kb_down_event, kb_up_event]).unwrap();
                        }
                    } else {
                        println!("{msg}");
                    }

                    sm.set_emitted(true);
                } else {
                    // println!("{}", sm.output());

                    emit_only_on_key_up_experiment(value, code, &mut virtual_device, *sm.emitted());
                }
            }
        }
    }
}

fn emit_only_on_key_up_experiment(
    value: i32,
    code: u16,
    virtual_device: &mut evdev::uinput::VirtualDevice,
    emitted: bool,
) {
    let key_codes_to_skip: Vec<u16> = vec![14, 29, 42, 54, 56, 97, 100, 125, 126];

    if key_codes_to_skip.contains(&code) {
        let event = event_from_code(code, value);
        virtual_device.emit(&[event]).unwrap();
    }

    if !key_codes_to_skip.contains(&code) && value == 0 && !emitted {
        let kb_down_event = event_from_code(code, 1);
        let kb_up_event = event_from_code(code, 0);

        virtual_device.emit(&[kb_down_event, kb_up_event]).unwrap();
    }
}

fn intercept(rx: Sender<TransmitSignal>, device_alias: &str, device_path: &str) {
    let device_alias = device_alias.to_string();

    let mut d = devices::input::from_path(device_path);
    match d.grab() {
        Ok(_) => println!("Grabbed {device_alias} {device_path} SUCCESSFULLY"),
        Err(err) => {
            println!("FAILED TO GRAB {device_alias} {device_path},\n{err},\n------------------");
        }
    }

    thread::spawn(move || loop {
        match d.fetch_events() {
            Err(err) => println!("Error fetching events. {err}"),
            Ok(events) => {
                for ev in events {
                    if ev.is_type_key() {
                        rx.send(TransmitSignal::Key(
                            device_alias.to_string(),
                            ev.code(),
                            ev.value(),
                            ev.timestamp(),
                        ))
                        .unwrap();
                    }
                }
            }
        }
    });
}
