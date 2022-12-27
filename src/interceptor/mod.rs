use std::{
    collections::HashMap,
    sync::mpsc::{self, Sender},
    thread,
    time::SystemTime,
};

use crate::devices::{self, input::EventKindCheck};

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
    let alias_map = mock_device_alias();

    // ----------------------------------------------------------------

    let (tx, rx) = mpsc::channel();

    for (device_alias, device_path) in alias_map {
        intercept(tx.clone(), device_alias, device_path);
    }

    // ----------------------------------------------------------------

    // let mut virtual_device = devices::output::new().unwrap();

    for signal in rx {
        match signal {
            TransmitSignal::Key(device_alias, code, value, _timestamp) => {
                println!("{device_alias} {code} {value}");
            }
        }
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
