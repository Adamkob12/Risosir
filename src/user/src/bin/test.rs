#![no_std]
#![no_main]

use core::{arch::asm, panic::PanicInfo};

#[no_mangle]
fn maine() {
    hi();
    unsafe { asm!("li s1, 69") };

    loop {}
}

fn hi() {
    unsafe { asm!("li s2, 69") };
    bye();
}

fn bye() {
    unsafe { asm!("li s3, 69") };
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {}
}
