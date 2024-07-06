#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]
#![feature(asm_const)]
#![feature(fn_align)]

pub mod arch;
pub mod entry;
pub mod kernelvec;
pub mod param;
pub mod start;
pub mod trap;
