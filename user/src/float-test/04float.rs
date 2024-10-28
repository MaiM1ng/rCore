#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

#[no_mangle]
fn main() -> isize {
    let a = 1.0;
    let b = 1.0;
    let mut c = 0.0;
    for _ in 0..500 {
        c = a + b + c;
        println!("c = {}", c);
    }
    0
}
