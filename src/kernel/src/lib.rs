#![feature(asm_const)]
#![allow(static_mut_refs)]
#![feature(naked_functions)]
#![feature(new_uninit)]
#![feature(fn_align)]
#![feature(panic_info_message)]
#![feature(ascii_char)]
#![feature(ascii_char_variants)]
#![feature(riscv_ext_intrinsics)]
#![no_std]
#![no_main]

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    use core::hint;
    // unsafe {
    //     UART.force_unlock();
    //     CONSOLE.force_unlock();
    //     let mut uart = UART.lock();

    //     uart.write_chars(b"\nPANIC: ");
    //     if let Some(msg) = info.message().as_str() {
    //         uart.write_chars(msg.as_bytes());
    //     } else {
    //         uart.write_chars(b"X");
    //     }
    //     uart.write_chars(b"\nFILE: ");
    //     uart.write_chars(info.location().unwrap().file().as_bytes());
    //     uart.write_chars(b"\nLINE: ");
    //     let mut line = info.location().unwrap().line();
    //     while line != 0 {
    //         uart.put_char((line % 10) as u8 + 48);
    //         line /= 10;
    //     }
    //     uart.put_char(b'\n');
    // };

    cprintln!("Encountered Panic (tp={}): {:#}", tp::read(), info);
    loop {
        hint::spin_loop();
    }
}

extern crate alloc;

pub mod arch;
pub mod console;
pub mod cpu;
pub mod elf_parse;
pub mod entry;
pub mod files;
pub mod kernelvec;
pub mod keyboard;
pub mod mem;
pub mod memlayout;
pub mod param;
pub mod plic;
pub mod proc;
pub mod scheduler;
pub mod start;
pub mod trampoline;
pub mod trap;
pub mod uart;
pub mod virtio;

use arch::registers::tp;
pub use console::*;
use core::panic::PanicInfo;

extern "C" {
    fn end();
    fn etext();
}

/// *Includes* the kernel's stack(s) (one stack per hart)
pub fn end_of_kernel_data_section() -> usize {
    end as usize
}

/// *Doesn't Include* kernel's stack(s), only includes "text" memory section
/// (includes trampoline)
pub fn end_of_kernel_code_section() -> usize {
    etext as usize
}
