use std::process::Command;

use crate::{
    devices::output::virtual_event, event_processor::sequence_manager::SequenceManager,
    stuffs::key_code::KeyCode,
};

pub enum Output {
    Map(&'static str),
    Cmd(&'static str, Vec<&'static str>),
}

pub fn emit_mapped_key(
    key: &str,
    sm: &SequenceManager,
    virtual_device: &mut evdev::uinput::VirtualDevice,
) {
    let code = KeyCode::from(key).0;
    if !sm.emitted() {
        virtual_device
            .emit(&[virtual_event(code, 1), virtual_event(code, 0)])
            .unwrap();
    }
}

pub fn emit_cmd(cmd: &str, args: &[&str], sm: &SequenceManager) {
    if !sm.emitted() {
        Command::new(cmd).args(args).spawn().ok();
    }
}
