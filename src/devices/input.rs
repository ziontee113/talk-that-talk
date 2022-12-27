use evdev::{Device, InputEvent, InputEventKind};

pub trait EventKindCheck {
    fn is_type_key(&self) -> bool;
}

impl EventKindCheck for InputEvent {
    fn is_type_key(&self) -> bool {
        matches!(&self.kind(), InputEventKind::Key(_))
    }
}

pub fn from_path(path: &str) -> Device {
    let devices = evdev::enumerate().map(|t| t.1).collect::<Vec<_>>();
    let mut device_index = 0;
    for (i, d) in devices.iter().enumerate() {
        if d.physical_path().is_some() && d.physical_path().unwrap() == path {
            device_index = i;
            break;
        }
    }
    devices.into_iter().nth(device_index).unwrap()
}

#[allow(dead_code)]
pub fn print_paths() {
    let devices = evdev::enumerate().map(|t| t.1).collect::<Vec<_>>();
    for d in &devices {
        if let Some(path) = d.physical_path() {
            let name = d.name().unwrap();
            println!("{name} = {path}");
        }
    }
}
