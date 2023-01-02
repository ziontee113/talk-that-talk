mod rule_output;

use std::{
    collections::HashMap,
    sync::mpsc::{self, Sender},
    thread,
    time::SystemTime,
};

use crate::{
    devices::{self, input::EventKindCheck, output::virtual_event},
    event_processor::sequence_manager::SequenceManager,
    stuffs::{key_identifier::KeyIdentifier, keyboard::Keyboard, keyboard_event::KeyboardEvent},
};

use self::rule_output::{emit_cmd, emit_mapped_key, emit_sequence, send_signal_to_neovim, Output};

pub enum TransmitSignal {
    Key(String, u16, i32, SystemTime),
    NeovimTCPPort(String),
}

// for development purposes only
fn mock_keyboard_devices() -> Vec<Keyboard> {
    vec![
        Keyboard::new("L1", "Left Keyboard", "usb-0000:00:1d.0-1.5.1.4/input0"),
        Keyboard::new("R1", "Right Keyboard", "usb-0000:00:1d.0-1.5.2/input0"),
    ]
}

// for development purposes only
fn create_mock_ruleset() -> HashMap<&'static str, Output> {
    HashMap::from([
        // Escape Key
        ("L1 CAPSLOCK", Output::Map("Esc")),
        // Arrow Keys
        ("L1 CAPSLOCK, R1 H", Output::Map("Left")),
        ("L1 CAPSLOCK, R1 J", Output::Map("Down")),
        ("L1 CAPSLOCK, R1 K", Output::Map("Up")),
        ("L1 CAPSLOCK, R1 L", Output::Map("Right")),
        // Playback Keys
        ("L1 H, R1 J", Output::Map("VolumeDown")),
        ("L1 H, R1 K", Output::Map("VolumeUp")),
        ("L1 H, R1 P", Output::Map("PreviousSong")),
        ("L1 H, R1 N", Output::Map("NextSong")),
        ("L1 H, R1 I", Output::Map("PlayPause")),
        // Cmd Test
        ("L1 E, R1 K", Output::Cmd("kitty", vec![])),
        ("L1 E, L1 F", Output::Cmd("firefox", vec![])),
        ("L1 E, R1 K, R1 J", Output::Cmd("gedit", vec![])),
        // Remap Right Alt to <C-F1>
        (
            "R1 RIGHTALT",
            Output::Sequence(vec![("LeftCtrl", 1), ("F1", 1), ("F1", 0), ("LeftCtrl", 0)]),
        ),
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
    let mut nvim_port = String::new();

    crate::http_server::start_server(tx);

    // Interception
    let mut virtual_device = devices::output::new().unwrap();
    let mut sm = SequenceManager::new();

    for signal in rx {
        match signal {
            TransmitSignal::NeovimTCPPort(port) => {
                nvim_port = port;
            }
            TransmitSignal::Key(device_alias, code, value, timestamp) => {
                if let Some(device) = keyboard_devices.iter().find(|d| *d.alias() == device_alias) {
                    let key = KeyIdentifier::new(device, code);
                    let event = KeyboardEvent::new(key, value, timestamp);

                    sm.receive(event);

                    // FRAUD_START:
                    let get_rule_from_ruleset = ruleset.get(sm.output().as_str());
                    // EXPLAIN_THIS:
                    if let Some(rule) = get_rule_from_ruleset {
                        match rule {
                            Output::Map(mapping) => {
                                emit_mapped_key(mapping, &sm, &mut virtual_device);
                            }
                            Output::Cmd(cmd, args) => emit_cmd(cmd, args, &sm),
                            Output::Sequence(sequence) => {
                                emit_sequence(sequence, &mut virtual_device);
                            }
                        }

                        sm.set_emitted(true);
                    }

                    // AND_THIS:
                    if !sm.emitted() && sm.is_combined() {
                        let modifiers: Vec<u16> = vec![14, 29, 42, 54, 56, 97, 125, 126];

                        if !modifiers.contains(&sm.first_code()) {
                            send_signal_to_neovim(&nvim_port, sm.output());
                            sm.set_emitted(true);
                        }
                    }

                    // AND_THIS:
                    if !sm.emitted() {
                        emit_only_on_key_up_experiment(value, code, &mut virtual_device, &sm);
                    }
                    // FRAUD_END:
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
    let modifiers: Vec<u16> = vec![14, 29, 42, 54, 56, 97, 125, 126];
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
