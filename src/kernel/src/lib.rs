#![no_std]
#![no_main]
#![feature(asm_const)]
#![feature(fn_align)]
#![feature(ascii_char)]
#![feature(ascii_char_variants)]

extern crate alloc;

pub mod arch;
pub mod console;
pub mod entry;
pub mod file;
pub mod kernelvec;
pub mod mem;
pub mod param;
pub mod proc;
pub mod start;
pub mod trap;
pub mod uart;

pub use console::*;

extern "C" {
    static end: u8;
}

pub fn end_of_kernel_code_address() -> usize {
    unsafe { &end as *const u8 as usize }
}
