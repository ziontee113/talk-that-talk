use std::io;

use evdev::{
    uinput::{VirtualDevice, VirtualDeviceBuilder},
    AttributeSet, EventType, InputEvent, Key,
};

pub fn new() -> Result<VirtualDevice, io::Error> {
    let keys: AttributeSet<Key> = (1..248).map(Key::new).collect();

    VirtualDeviceBuilder::new()?
        .name("Virtual Keyboard")
        .with_keys(&keys)?
        .build()
}

pub fn virtual_event(key_code: u16, key_value: i32) -> InputEvent {
    InputEvent::new(EventType::KEY, key_code, key_value)
}
