use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::sync::mpsc::Sender;
use std::thread;

use serde::Deserialize;

use crate::interceptor::TransmitSignal;

#[derive(Deserialize, Debug)]
struct NeovimInstance {
    cwd: String,
}

pub fn start_server(tx: Sender<TransmitSignal>) {
    thread::spawn(move || {
        let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

        for stream in listener.incoming() {
            if let Some(cwd) = handle_stream(stream.unwrap()) {
                tx.send(TransmitSignal::NeovimCWD(cwd)).unwrap();
            }
        }
    });
}

fn handle_stream(mut stream: TcpStream) -> Option<String> {
    let mut buffer = [0; 1024];
    let _ = stream.read(&mut buffer).unwrap();

    let result = String::from_utf8_lossy(&buffer[..]); // read to buffer
    let result = result.lines().nth(7).unwrap(); // get content from line 7
    let result = result.replace('\0', ""); // remove \0 characters

    match serde_json::from_str::<NeovimInstance>(&result) {
        Ok(i) => Some(i.cwd),
        Err(_) => None,
    }
}
