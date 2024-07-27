#![no_std]
#![no_main]

// use fs::FILES;
use kernel::*;

#[no_mangle]
fn maine() {
    cprintln!("{}", 1 + 1);
}
