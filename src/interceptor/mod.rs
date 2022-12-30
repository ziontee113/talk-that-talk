use std::{
    collections::HashMap,
    sync::mpsc::{self, Sender},
    thread,
    time::SystemTime,
};

use crate::{
    devices::{self, input::EventKindCheck, output::virtual_event},
    event_processor::sequence_manager::SequenceManager,
    stuffs::{
        key_code::KeyCode, key_identifier::KeyIdentifier, keyboard::Keyboard,
        keyboard_event::KeyboardEvent,
    },
};

pub enum TransmitSignal {
    Key(String, u16, i32, SystemTime),
    NeovimCWD(String),
}

// for development purposes only
fn mock_keyboard_devices() -> Vec<Keyboard> {
    vec![
        Keyboard::new("L1", "Left Keyboard", "usb-0000:00:1d.0-1.5.1.4/input0"),
        Keyboard::new("R1", "Right Keyboard", "usb-0000:00:1d.0-1.5.2/input0"),
    ]
}

// for development purposes only
fn create_mock_ruleset() -> HashMap<&'static str, &'static str> {
    HashMap::from([
        ("L1 CAPSLOCK Down", "MAP_TO: ESC"),
        ("L1 CAPSLOCK Down, R1 H Down", "MAP_TO: Left"),
        ("L1 CAPSLOCK Down, R1 J Down", "MAP_TO: Down"),
        ("L1 CAPSLOCK Down, R1 K Down", "MAP_TO: Up"),
        ("L1 CAPSLOCK Down, R1 L Down", "MAP_TO: Right"),
        ("L1 H Down, R1 J Down", "MAP_TO: VolumeDown"),
        ("L1 H Down, R1 K Down", "MAP_TO: VolumeUp"),
        ("L1 H Down, R1 P Down", "MAP_TO: PreviousSong"),
        ("L1 H Down, R1 N Down", "MAP_TO: NextSong"),
        ("L1 H Down, R1 I Down", "MAP_TO: PlayPause"),
    ])
}

pub fn start() {
    // Development Variables
    let keyboard_devices = mock_keyboard_devices();
    let ruleset = create_mock_ruleset();

    // Message Channels
    let (tx, rx) = mpsc::channel();

    for keyboard in &keyboard_devices {
        intercept(tx.clone(), keyboard);
    }

    // HTTP server
    crate::http_server::start_server(tx);

    // Interception
    let mut virtual_device = devices::output::new().unwrap();
    let mut sm = SequenceManager::new();

    for signal in rx {
        match signal {
            TransmitSignal::NeovimCWD(cwd) => {
                println!("Neovim instance at: {cwd}");
            }
            TransmitSignal::Key(device_alias, code, value, timestamp) => {
                if let Some(device) = keyboard_devices.iter().find(|d| *d.alias() == device_alias) {
                    let key = KeyIdentifier::new(device, code);
                    let event = KeyboardEvent::new(key, value, timestamp);

                    sm.receive(event);

                    // FRAUD:
                    let get_rule_from_ruleset = ruleset.get(sm.output().as_str());
                    if let Some(rule) = get_rule_from_ruleset {
                        let emit_pattern = "MAP_TO: ";
                        let split: Vec<&str> = rule.split(emit_pattern).collect();

                        if rule.contains(emit_pattern) {
                            emit_mapped_key(&split, &sm, &mut virtual_device);
                        } else {
                            println!("{rule}");
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

fn emit_mapped_key(
    split: &[&str],
    sm: &SequenceManager,
    virtual_device: &mut evdev::uinput::VirtualDevice,
) {
    let code = KeyCode::from(*split.last().unwrap()).0;
    if !sm.emitted() {
        virtual_device
            .emit(&[virtual_event(code, 1), virtual_event(code, 0)])
            .unwrap();
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
        let event = virtual_event(code, value);
        virtual_device.emit(&[event]).unwrap();
    }

    if !modifiers.contains(&code) && value == 0 && !sm.emitted() {
        // handle down events
        let mut events = vec![];
        for modifier_code in sm.modifiers() {
            events.push(virtual_event(*modifier_code, 1));
        }
        events.push(virtual_event(code, 1));

        // handle up events
        let mut up_events = vec![];
        for modifier_code in sm.modifiers() {
            up_events.push(virtual_event(*modifier_code, 0));
        }
        up_events.push(virtual_event(code, 0));

        // append and emit
        events.append(&mut up_events);
        virtual_device.emit(&events).unwrap();
    }
}

fn intercept(tx: Sender<TransmitSignal>, device: &Keyboard) {
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
                        tx.send(TransmitSignal::Key(
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
