#![no_std]
#![no_main]

use core::{arch::asm, panic::PanicInfo};

#[no_mangle]
fn maine() {
    let mut a = 0;
    unsafe {
        asm!("ecall");
        loop {
            a += 100;
            asm!("mv s2, {x}", x = in(reg) a);
            asm!("addi s1, s1, 1");
        }
    }
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {}
}
