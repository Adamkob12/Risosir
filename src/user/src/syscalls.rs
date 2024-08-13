use core::arch::asm;
use kernel::syscall::*;

pub fn print(x: &str) {
    unsafe { sys_print(x.as_ptr(), x.len()) }
}

pub fn exit(exit_code: usize) {
    unsafe { sys_exit(exit_code) };
}

#[inline(never)]
unsafe extern "C" fn sys_print(_ptr: *const u8, _len: usize) {
    asm!("li a6, {sys}", sys = const PRINT_SYSCALL);
    asm!("ecall");
}

#[inline(never)]
unsafe extern "C" fn sys_exit(_exit_code: usize) {
    asm!("li a6, {sys}", sys = const EXIT_SYSCALL);
    asm!("ecall");
}
