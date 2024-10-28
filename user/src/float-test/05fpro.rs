#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

#[no_mangle]
fn main() -> isize {
    let mut a = 1.0;
    let b = 1.5;

    for _ in 0..200 {
        a = a * b;
        println!("fpro a = {}", a);
    }
    0
}
