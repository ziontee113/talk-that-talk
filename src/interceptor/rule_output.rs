use std::{io::Read, process::Command};

use evdev::uinput::VirtualDevice;

use crate::{
    devices::output::virtual_event, event_processor::sequence_manager::SequenceManager,
    stuffs::key_code::KeyCode,
};

pub enum Output {
    Map(&'static str),
    Cmd(&'static str, Vec<&'static str>),
    Nvim(&'static str),
    Sequence(Vec<(&'static str, i32)>),
}

pub fn emit_mapped_key(key: &str, sm: &SequenceManager, virtual_device: &mut VirtualDevice) {
    let code = KeyCode::from(key).0;
    if !sm.emitted() {
        virtual_device
            .emit(&[virtual_event(code, 1), virtual_event(code, 0)])
            .unwrap();
    }
}

pub fn emit_nvim_msg<S: Into<String>>(cwd: &str, msg: S) {
    let pipe = cwd.to_owned() + "/nvim.pipe";

    Command::new("nvim")
        .args(["--server", &pipe, "--remote-send", &msg.into()])
        .spawn()
        .ok();
}

pub fn emit_cmd(cmd: &str, args: &[&str], sm: &SequenceManager) {
    if !sm.emitted() {
        Command::new(cmd).args(args).spawn().ok();
    }
}

pub fn emit_sequence(sequence: &Vec<(&str, i32)>, virtual_device: &mut VirtualDevice) {
    for e in sequence {
        let code = KeyCode::from(e.0).0;
        let event = virtual_event(code, e.1);

        virtual_device.emit(&[event]).unwrap();
    }
}

pub fn send_back(port: &str, msg: &str) {
    let address = format!("localhost:{port}");

    match std::net::TcpStream::connect(address.clone()) {
        Ok(mut stream) => {
            println!("Successfully connected to server in port {address}");

            let msg = msg.as_bytes();

            std::io::Write::write(&mut stream, msg).unwrap();
            println!("Sent Hello, awaiting reply...");

            let mut buffer = [0; 1024];
            let read_result = stream.read(&mut buffer);

            match read_result {
                Ok(msg_length) => {
                    let buffer = &buffer[..msg_length];

                    if buffer == msg {
                        println!("Reply is ok!");
                    } else {
                        let text = std::str::from_utf8(buffer).unwrap();
                        println!("Unexpected reply: {text}");

                        println!(
                            "{} vs {}",
                            std::str::from_utf8(buffer).unwrap(),
                            std::str::from_utf8(msg).unwrap()
                        );
                    }
                }
                Err(e) => {
                    println!("Failed to receive data: {e}");
                }
            }
        }
        Err(e) => {
            println!("tried to connect to: {address}");
            println!("Failed to connect: {e}");
        }
    }
    println!("Terminated.");
}
