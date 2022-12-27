use std::{
    collections::HashMap,
    sync::mpsc::{self, Sender},
    thread,
    time::SystemTime,
};

use crate::devices::{self, input::EventKindCheck, output::event_from_code};

enum TransmitSignal {
    Key(String, u16, i32, SystemTime),
}

// for development purposes only
pub fn mock_device_alias() -> HashMap<&'static str, &'static str> {
    HashMap::from([
        ("L1", "usb-0000:00:1d.0-1.5.1.4/input0"),
        ("R1", "usb-0000:00:1d.0-1.5.2/input0"),
    ])
}

pub fn start() {
    // Development Variables
    let alias_map = mock_device_alias();

    // Message Channels
    let (tx, rx) = mpsc::channel();

    for (device_alias, device_path) in alias_map {
        intercept(tx.clone(), device_alias, device_path);
    }

    // Interception
    let mut virtual_device = devices::output::new().unwrap();

    for signal in rx {
        match signal {
            TransmitSignal::Key(_device_alias, code, value, _timestamp) => {
                emit_only_on_key_up_experiment(value, code, &mut virtual_device);
            }
        }
    }
}

fn emit_only_on_key_up_experiment(
    value: i32,
    code: u16,
    virtual_device: &mut evdev::uinput::VirtualDevice,
) {
    let keys_to_skip: Vec<u16> = vec![14, 29, 42, 54, 56, 97, 100, 125, 126];

    if keys_to_skip.contains(&code) {
        let event = event_from_code(code, value);
        virtual_device.emit(&[event]).unwrap();
    }

    if !keys_to_skip.contains(&code) && value == 0 {
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
