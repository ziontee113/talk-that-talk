#![allow(dead_code, unused_macros)]
#![warn(clippy::pedantic)]

#[macro_use]
extern crate getset;

mod devices;
mod interceptor;
mod stuffs;
mod test_utilities;

fn main() {
    interceptor::start();
}
