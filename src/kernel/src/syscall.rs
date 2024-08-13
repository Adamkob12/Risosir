use core::{mem::transmute, slice, str};

use fs::FileId;

use crate::{
    cprint, cprintln,
    cpu::cproc,
    mem::{
        paging::{translate, PageTableLevel},
        virtual_mem::{PTEFlags, VirtAddr},
    },
};

pub const READ_SYSCALL: usize = 10;
pub const PRINT_SYSCALL: usize = 11;
pub const EXIT_SYSCALL: usize = 12;

pub unsafe fn syscall() {
    let a0 = cproc().trapframe().a0;

    let a1 = cproc().trapframe().a1;
    let a2 = cproc().trapframe().a2;
    let _a3 = cproc().trapframe().a3;
    let _a4 = cproc().trapframe().a4;
    let _a5 = cproc().trapframe().a5;
    let a6 = cproc().trapframe().a6;

    match a6 {
        READ_SYSCALL => read_syscall(a0 as u16, slice::from_raw_parts_mut(transmute(a1), a2)),
        PRINT_SYSCALL => {
            let addr = translate(
                cproc().pagetable(),
                VirtAddr::from_raw(a0 as u64),
                PageTableLevel::L2,
                PTEFlags::valid().readable().userable(),
            );
            let len = a1;
            #[cfg(debug_assertions)]
            cprintln!(
                "executing print syscall | addr={:#x}, len={}",
                addr.as_u64(),
                len
            );
            print_syscall(str::from_raw_parts(addr.as_u64() as *const u8, a1));
        }
        EXIT_SYSCALL => {
            exit_syscall(a0);
        }
        syscall => panic!("Unrecognized Syscall: {syscall}"),
    }
}

pub fn read_syscall(_file_id: FileId, _buff: &mut [u8]) {
    todo!()
}

pub fn print_syscall(to_print: &str) {
    cprint!("{}", to_print);
}

pub fn exit_syscall(exit_code: usize) {
    cproc().exit(exit_code);
}
