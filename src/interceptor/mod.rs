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
fn mock_keyboard_devices() -> Vec<Keyboard> {
    vec![
        Keyboard::new("L1", "Left Keyboard", "usb-0000:00:1d.0-1.5.1.4/input0"),
        Keyboard::new("R1", "Right Keyboard", "usb-0000:00:1d.0-1.5.2/input0"),
    ]
}

// for development purposes only
fn rules_to_print() -> HashMap<&'static str, &'static str> {
    HashMap::from([
        ("L1 CAPSLOCK Down", "MAP_CODE: 1"),
        ("L1 CAPSLOCK Down, R1 H Down", "MAP_CODE: 105"),
        ("L1 CAPSLOCK Down, R1 J Down", "MAP_CODE: 108"),
        ("L1 CAPSLOCK Down, R1 K Down", "MAP_CODE: 103"),
        ("L1 CAPSLOCK Down, R1 L Down", "MAP_CODE: 106"),
        ("L1 H Down, R1 J Down", "MAP_CODE: 114"),
        ("L1 H Down, R1 K Down", "MAP_CODE: 115"),
    ])
}

pub fn start() {
    // Development Variables
    let keyboard_devices = mock_keyboard_devices();
    let rules_to_print = rules_to_print();

    // Message Channels
    let (tx, rx) = mpsc::channel();

    for keyboard in &keyboard_devices {
        intercept(tx.clone(), keyboard);
    }

    // Interception
    let mut virtual_device = devices::output::new().unwrap();
    let mut sm = SequenceManager::new();

    for signal in rx {
        match signal {
            TransmitSignal::Key(device_alias, code, value, timestamp) => {
                if let Some(device) = keyboard_devices.iter().find(|d| *d.alias() == device_alias) {
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
                        emit_only_on_key_up_experiment(value, code, &mut virtual_device, &sm);
                    }
                }
            }
        }
    }
}

fn emit_only_on_key_up_experiment(
    value: i32,
    code: u16,
    virtual_device: &mut evdev::uinput::VirtualDevice,
    sm: &SequenceManager,
) {
    let modifiers: Vec<u16> = vec![14, 29, 42, 54, 56, 97, 100, 125, 126];
    let ignore_list: Vec<u16> = vec![58];

    if ignore_list.contains(&code) {
        return;
    }

    if modifiers.contains(&code) {
        let event = event_from_code(code, value);
        virtual_device.emit(&[event]).unwrap();
    }

    if !modifiers.contains(&code) && value == 0 && !sm.emitted() {
        // handle down events
        let mut events = vec![];
        for modifier_code in sm.modifiers() {
            events.push(event_from_code(*modifier_code, 1));
        }
        events.push(event_from_code(code, 1));

        // handle up events
        let mut up_events = vec![];
        for modifier_code in sm.modifiers() {
            up_events.push(event_from_code(*modifier_code, 0));
        }
        up_events.push(event_from_code(code, 0));

        // append and emit
        events.append(&mut up_events);
        virtual_device.emit(&events).unwrap();
    }
}

fn intercept(rx: Sender<TransmitSignal>, device: &Keyboard) {
    let alias = device.alias().clone();
    let path = device.path();

    let mut d = devices::input::from_path(path);
    match d.grab() {
        Ok(_) => println!("Grabbed {alias} {path} SUCCESSFULLY"),
        Err(err) => {
            println!("FAILED TO GRAB {alias} {path},\n{err},\n------------------",);
        }
    }

    thread::spawn(move || loop {
        match d.fetch_events() {
            Err(err) => println!("Error fetching events. {err}"),
            Ok(events) => {
                for ev in events {
                    if ev.is_type_key() {
                        rx.send(TransmitSignal::Key(
                            alias.to_string(),
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
