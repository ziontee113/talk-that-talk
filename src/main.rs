#![allow(dead_code, unused_macros)]
#![warn(clippy::pedantic)]
#![feature(drain_filter)]

#[macro_use]
extern crate getset;

mod devices;
mod event_processor;
mod http_server;
mod interceptor;
mod stuffs;
mod test_utilities;

fn main() {
    interceptor::start();
}
