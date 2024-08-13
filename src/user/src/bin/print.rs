#![no_std]
#![no_main]

use user::syscalls::{exit, print};

#[no_mangle]
fn maine() {
    print("Hi! (From User Mode)\n");
    exit(1);
}
