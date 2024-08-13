#![no_std]
#![no_main]

use user::syscalls::{exit, print};

#[no_mangle]
fn maine() {
    unsafe { print("Hi! (From User Mode)\n") };
    unsafe { exit(1) };
}
