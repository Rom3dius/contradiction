#![no_main]
#![no_std]

use risc0_zkvm::guest::env;
use core::convert::TryInto;

risc0_zkvm::guest::entry!(main);

fn main() {
    let side_a: u8 = env::read();
    let side_b: u32 = env::read();

    let hypotenuse = compute_hypotenuse(side_a, side_b);

    env::commit(&hypotenuse);
}

fn compute_hypotenuse(a: u32, b: u32) -> f64 {
    let a_squared = (a as f64).powi(2);
    let b_squared = (b as f64).powi(2);
    let sum = a_squared + b_squared;
    
    sum.sqrt()
}