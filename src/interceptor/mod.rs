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

use self::rule_output::{emit_cmd, emit_mapped_key, emit_nvim_msg, Output};

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
fn create_mock_ruleset() -> HashMap<&'static str, Output> {
    HashMap::from([
        // Escape Key
        ("L1 CAPSLOCK Down", Output::Map("Esc")),
        // Arrow Keys
        ("L1 CAPSLOCK Down, R1 H Down", Output::Map("Left")),
        ("L1 CAPSLOCK Down, R1 J Down", Output::Map("Down")),
        ("L1 CAPSLOCK Down, R1 K Down", Output::Map("Up")),
        ("L1 CAPSLOCK Down, R1 L Down", Output::Map("Right")),
        // Playback Keys
        ("L1 H Down, R1 J Down", Output::Map("VolumeDown")),
        ("L1 H Down, R1 K Down", Output::Map("VolumeUp")),
        ("L1 H Down, R1 P Down", Output::Map("PreviousSong")),
        ("L1 H Down, R1 N Down", Output::Map("NextSong")),
        ("L1 H Down, R1 I Down", Output::Map("PlayPause")),
        // Cmd Test
        ("L1 E Down, R1 K Down", Output::Cmd("kitty", vec![])),
        ("L1 E Down, L1 F Down", Output::Cmd("firefox", vec![])),
        (
            "L1 E Down, R1 K Down, R1 J Down",
            Output::Cmd("gedit", vec![]),
        ),
        // Nvim Testing
        ("L1 A Down, R1 J Down", Output::Nvim("4j")),
        ("L1 A Down, R1 K Down", Output::Nvim("4k")),
        ("L1 A Down, R1 H Down", Output::Nvim("8k")),
        ("L1 A Down, R1 L Down", Output::Nvim("8j")),
        // Testing the waters, why does this work?
        ("L1 S Down, R1 O Down", Output::Nvim("4j")),
        ("R1 O Down, R1 M Down", Output::Nvim("4k")),
        ("L1 S Down, R1 M Down", Output::Nvim("4j")),
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
    let mut current_nvim_directory = String::new();
    crate::http_server::start_server(tx);

    // Interception
    let mut virtual_device = devices::output::new().unwrap();
    let mut sm = SequenceManager::new();

    for signal in rx {
        match signal {
            TransmitSignal::NeovimCWD(cwd) => {
                current_nvim_directory = cwd;
            }
            TransmitSignal::Key(device_alias, code, value, timestamp) => {
                if let Some(device) = keyboard_devices.iter().find(|d| *d.alias() == device_alias) {
                    let key = KeyIdentifier::new(device, code);
                    let event = KeyboardEvent::new(key, value, timestamp);

                    sm.receive(event);

                    // FRAUD_START:
                    let get_rule_from_ruleset = ruleset.get(sm.output().as_str());
                    if let Some(rule) = get_rule_from_ruleset {
                        match rule {
                            Output::Map(mapping) => {
                                emit_mapped_key(mapping, &sm, &mut virtual_device);
                            }
                            Output::Cmd(cmd, args) => emit_cmd(cmd, args, &sm),
                            Output::Nvim(msg) => {
                                emit_nvim_msg(&current_nvim_directory, *msg);
                            }
                        }

                        sm.set_emitted(true);
                    }

                    if !sm.emitted() && sm.output().contains(',') {
                        let modifiers: Vec<u16> = vec![14, 29, 42, 54, 56, 97, 100, 125, 126];
                        let first_code = sm.sequence().first().unwrap().key().code().0;

                        if !modifiers.contains(&first_code) {
                            println!("{}", sm.output());
                            emit_nvim_msg(
                                &current_nvim_directory,
                                format!("<Plug>{}", sm.output()),
                            );
                            sm.set_emitted(true);
                        }
                    }

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
