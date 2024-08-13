pub mod exception;
pub mod interrupt;

use crate::arch::interrupts::*;
use crate::arch::registers::tp;
use crate::cprintln;
use crate::cpu::cproc;
use crate::mem::paging::{make_satp, translate};
use crate::mem::virtual_mem::{PTEFlags, VirtAddr};
use crate::memlayout::TRAMPOLINE_VADDR;
use crate::param::STACK_SIZE;
use crate::proc::ProcStatus;
use crate::trampoline::trampoline;
use crate::{
    memlayout::{UART_IRQ, VIRTIO0_IRQ},
    plic::{plic_claim, plic_complete},
    proc::cpuid,
    uart::uart_interrupt,
    virtio::virtio_intr,
};
use core::arch::asm;
use core::sync::atomic::Ordering;
pub use exception::*;
pub use interrupt::*;
use riscv::register::scause::Exception;
use riscv::register::{satp, sepc, sstatus, stvec};
use riscv::register::{
    scause::{self, Interrupt},
    stval,
};

extern "C" {
    fn uservec() -> !;
    fn userret(satp: usize) -> !;
}

/// The first function that should be executed when a new process in created.
/// This function should be called while still in s-mode, and with the kernel page table.
pub fn user_proc_entry() {
    s_disable();
    let proc = cproc();
    cprintln!(
        "Address for entry point for new process {} in Kernel Ram: {:#x}",
        proc.name(),
        translate(
            &proc.pagetable(),
            VirtAddr::from_raw(0x1000),
            crate::mem::paging::PageTableLevel::L2,
            PTEFlags::valid().executable().readable().writable(),
        )
        .as_u64()
    );
    user_trap_return();
}

fn user_trap_return() -> ! {
    s_disable();
    let proc = cproc();

    #[cfg(debug_assertions)]
    assert_eq!(proc.status.load(Ordering::SeqCst), ProcStatus::Running);

    // SAFETY: if the process is running, the trapframe of the process can only be accessed by the CPU that's running the process.
    let tf = unsafe { proc.trapframe.as_mut().unwrap() };
    tf.kernel_satp = satp::read().bits();
    tf.kernel_hartid = tp::read();
    tf.kernel_sp = proc.kernel_stack as usize + STACK_SIZE;
    tf.kernel_trap = usertrap as usize;
    tf.a4 = 0x69;

    unsafe {
        sstatus::set_spp(sstatus::SPP::User);
        sstatus::set_spie();
        sepc::write(tf.epc);
        stvec::write(
            TRAMPOLINE_VADDR + (uservec as usize - trampoline as usize),
            stvec::TrapMode::Direct,
        );

        let userret_addr = TRAMPOLINE_VADDR + (userret as usize - trampoline as usize);
        let userret_fn: extern "C" fn(usize) -> ! = core::mem::transmute(userret_addr);

        userret_fn(make_satp(proc.page_table as usize))
    }
}

fn device_interrupt(hart_id: usize) {
    if let Some(plic_irq) = plic_claim(hart_id) {
        match plic_irq {
            VIRTIO0_IRQ => {
                cprintln!("virtio");
                virtio_intr();
            }
            UART_IRQ => {
                uart_interrupt();
            }
            id => {
                panic!("PLIC - Unrecognized interrupt: {id}");
            }
        }
        plic_complete(hart_id, plic_irq);
    }
}

#[no_mangle]
pub unsafe extern "C" fn usertrap() {
    let scause = scause::read();
    let hart_id = cpuid();
    stvec::write(kernelvec as usize, stvec::TrapMode::Direct);
    unsafe { *cproc().trapframe }.epc = sepc::read();
    match scause.cause() {
        scause::Trap::Interrupt(int) => match int {
            Interrupt::SupervisorExternal => device_interrupt(hart_id),
            Interrupt::SupervisorSoft => {
                cprintln!("User Timer Int s1={}", cproc().trapframe().s1);
                let sip: usize;
                asm!("csrr {x}, sip", x = out(reg) sip);
                asm!("csrw sip, {x}", x = in(reg) (sip & !2));
            }
            int => {
                panic!("Unrecognized interrupt: {:#?}", int)
            }
        },
        scause::Trap::Exception(excp) => match excp {
            Exception::UserEnvCall => {
                cprintln!("ecall");
                unsafe { cproc().trapframe.as_mut().unwrap() }.epc += 4;
            }
            _ => panic!(
                "Unexpected Exception in User Mode: \n\tScause={:#b}\n\tStval={}",
                scause.bits(),
                stval::read(),
            ),
        },
    }

    user_trap_return();
}

#[no_mangle]
pub unsafe extern "C" fn kerneltrap() {
    let scause = scause::read();
    let hart_id = 0;
    let sepc = sepc::read();

    match scause.cause() {
        scause::Trap::Interrupt(int) => match int {
            Interrupt::SupervisorExternal => device_interrupt(hart_id),
            Interrupt::SupervisorSoft => {
                cprintln!("hai");
                let sip: usize;
                asm!("csrr {x}, sip", x = out(reg) sip);
                asm!("csrw sip, {x}", x = in(reg) (sip & !2));
            }
            int => {
                panic!("Unrecognized interrupt: {:#?}", int)
            }
        },
        scause::Trap::Exception(excp) => match excp {
            _ => panic!(
                "Unexpected Exception in Kernel: \n\tScause={:#b}\n\tStval={}",
                scause.bits(),
                stval::read(),
            ),
        },
    }

    sepc::write(sepc);
}

#[repr(align(16))]
#[no_mangle]
pub unsafe extern "C" fn kernelvec() -> ! {
    asm!(
        // make room to save registers.
        "addi sp, sp, -256",
        // save the registers.
        "sd ra, 0(sp)",
        "sd sp, 8(sp)",
        "sd gp, 16(sp)",
        "sd tp, 24(sp)",
        "sd t0, 32(sp)",
        "sd t1, 40(sp)",
        "sd t2, 48(sp)",
        "sd s0, 56(sp)",
        "sd s1, 64(sp)",
        "sd a0, 72(sp)",
        "sd a1, 80(sp)",
        "sd a2, 88(sp)",
        "sd a3, 96(sp)",
        "sd a4, 104(sp)",
        "sd a5, 112(sp)",
        "sd a6, 120(sp)",
        "sd a7, 128(sp)",
        "sd s2, 136(sp)",
        "sd s3, 144(sp)",
        "sd s4, 152(sp)",
        "sd s5, 160(sp)",
        "sd s6, 168(sp)",
        "sd s7, 176(sp)",
        "sd s8, 184(sp)",
        "sd s9, 192(sp)",
        "sd s10, 200(sp)",
        "sd s11, 208(sp)",
        "sd t3, 216(sp)",
        "sd t4, 224(sp)",
        "sd t5, 232(sp)",
        "sd t6, 240(sp)",
        // call the Rust trap handler in trap.rs
        "call kerneltrap",
        // restore registers.
        "ld ra, 0(sp)",
        "ld sp, 8(sp)",
        "ld gp, 16(sp)",
        // not this, in case we moved CPUs: ld tp, 24(sp)
        "ld t0, 32(sp)",
        "ld t1, 40(sp)",
        "ld t2, 48(sp)",
        "ld s0, 56(sp)",
        "ld s1, 64(sp)",
        "ld a0, 72(sp)",
        "ld a1, 80(sp)",
        "ld a2, 88(sp)",
        "ld a3, 96(sp)",
        "ld a4, 104(sp)",
        "ld a5, 112(sp)",
        "ld a6, 120(sp)",
        "ld a7, 128(sp)",
        "ld s2, 136(sp)",
        "ld s3, 144(sp)",
        "ld s4, 152(sp)",
        "ld s5, 160(sp)",
        "ld s6, 168(sp)",
        "ld s7, 176(sp)",
        "ld s8, 184(sp)",
        "ld s9, 192(sp)",
        "ld s10, 200(sp)",
        "ld s11, 208(sp)",
        "ld t3, 216(sp)",
        "ld t4, 224(sp)",
        "ld t5, 232(sp)",
        "ld t6, 240(sp)",
        "addi sp, sp, 256",
        // return to whatever we were doing in the kernel.
        "sret",
        options(noreturn)
    );
}
