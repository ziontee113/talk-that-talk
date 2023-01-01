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
        let listener = TcpListener::bind("0.0.0.0:3333").unwrap();

        for stream in listener.incoming() {
            let port = handle_stream(stream.unwrap());
            tx.send(TransmitSignal::NeovimTCPPort(port)).unwrap();
        }
    });
}

fn handle_stream(mut stream: TcpStream) -> String {
    let mut buffer = [0; 32];
    let read = stream.read(&mut buffer).unwrap();

    String::from_utf8_lossy(&buffer[..read]).to_string()
}
