//! Trap handling functionality
//!
//! For rCore, we have a single trap entry point, namely `__alltraps`. At
//! initialization in [`init()`], we set the `stvec` CSR to point to it.
//!
//! All traps go through `__alltraps`, which is defined in `trap.S`. The
//! assembly language code does just enough work restore the kernel space
//! context, ensuring that Rust code safely runs, and transfers control to
//! [`trap_handler()`].
//!
//! It then calls different functionality based on what exactly the exception
//! was. For example, timer interrupts trigger task preemption, and syscalls go
//! to [`syscall()`].

mod context;
/// check kernel interrupt
pub mod kernel_trap;

use crate::syscall::syscall;
use crate::task::{
    exit_current_and_run_next, suspend_current_and_run_next, update_current_task_kernel_time,
    update_current_task_user_time,
};
use crate::timer::set_next_trigger;
use core::arch::global_asm;
use kernel_trap::mark_kernel_interrupt_triggered;
use riscv::register::sstatus::{self};
use riscv::register::{
    mtvec::TrapMode,
    scause::{self, Exception, Interrupt, Trap},
    sie, stval, stvec,
};

global_asm!(include_str!("trap.S"));

/// initialize CSR `stvec` as the entry of `__alltraps`
pub fn init() {
    extern "C" {
        fn __alltraps();
    }
    unsafe {
        stvec::write(__alltraps as usize, TrapMode::Direct);
    }
}

/// timer interrupt enabled
pub fn enable_timer_interrupt() {
    unsafe {
        sie::set_stimer();
    }
}

#[no_mangle]
/// handle an interrupt, exception, or system call from user space
pub fn trap_handler(cx: &mut TrapContext) -> &mut TrapContext {
    // 根据进入内核前的特权等级判断是来自于谁的异常
    match sstatus::read().spp() {
        sstatus::SPP::User => user_trap_handler(cx),
        sstatus::SPP::Supervisor => kernel_trap_handler(cx),
    }
}

/// handle user trap
pub fn user_trap_handler(cx: &mut TrapContext) -> &mut TrapContext {
    // 更新用户态时间
    update_current_task_user_time();

    let scause = scause::read(); // get trap cause
    let stval = stval::read(); // get extra value
    match scause.cause() {
        Trap::Exception(Exception::UserEnvCall) => {
            cx.sepc += 4;
            cx.x[10] = syscall(cx.x[17], [cx.x[10], cx.x[11], cx.x[12]]) as usize;
        }
        Trap::Exception(Exception::StoreFault) | Trap::Exception(Exception::StorePageFault) => {
            println!("[kernel] PageFault in application, bad addr = {:#x}, bad instruction = {:#x}, kernel killed it.", stval, cx.sepc);
            exit_current_and_run_next();
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            println!("[kernel] IllegalInstruction in application, kernel killed it.");
            // println!("[Kernel] sepc = {:x}", sepc::read());
            exit_current_and_run_next();
        }
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            set_next_trigger();
            suspend_current_and_run_next();
        }
        _ => {
            panic!(
                "Unsupported trap {:?}, stval = {:#x}!",
                scause.cause(),
                stval
            );
        }
    }
    // TODO: 为什么返回值还是cx
    update_current_task_kernel_time();
    cx
}

/// handle kernel trap
pub fn kernel_trap_handler(cx: &mut TrapContext) -> &mut TrapContext {
    let scause = scause::read();
    let stval = stval::read();

    match scause.cause() {
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            println!("[Kernel] Kernel Interrupt: from time!");
            mark_kernel_interrupt_triggered();
            set_next_trigger();
        }
        Trap::Exception(Exception::StoreFault) | Trap::Exception(Exception::StorePageFault) => {
            panic!("[Kernel] PageFault in kernel, bad addr = {:#x}, bad instruction = {:#x}, kernel killed.", stval, cx.sepc);
        }
        _ => {
            panic!("[Kernel] Unknown kernel exception or interrupts");
        }
    }
    cx
}

pub use context::TrapContext;
