#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use core::arch::asm;

#[inline(never)]
#[no_mangle]
fn show_func_trace() {
    let mut fp: usize;
    unsafe {
        asm!(
            "mv {fp}, s0",
            fp = out(reg) fp,
        );
    }

    while fp != 0 {
        let ra = unsafe { *(fp as *const usize).offset(-1) };
        let old_sp = unsafe { *(fp as *const usize).offset(-2) };

        println!("ra = {:x}", ra);
        println!("old sp = {:x}", old_sp);
        println!("");
        fp = old_sp;
    }
}

#[inline(never)]
#[no_mangle]
fn f3() {
    show_func_trace();
}

#[inline(never)]
#[no_mangle]
fn f2() {
    f3();
}

#[inline(never)]
#[no_mangle]
fn f1() {
    f2();
}

#[no_mangle]
fn main() -> i32 {
    println!("====== Stack trace from chain ======\n");

    f1();

    0
}
