#![no_std]
#![no_main]

use core::{arch::asm, panic::PanicInfo};

#[no_mangle]
fn maine() {
    hi();
    unsafe { asm!("li t1, 69") };
}

fn hi() {
    unsafe { asm!("li t1, 69") };
    bye();
}

fn bye() {
    unsafe { asm!("li t1, 69") };
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {}
}
