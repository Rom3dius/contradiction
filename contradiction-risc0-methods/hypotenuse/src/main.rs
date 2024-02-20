#![no_main]
#![no_std]

use risc0_zkvm::guest::env;

risc0_zkvm::guest::entry!(main);

fn main() {
    let side_a: u32 = env::read();
    let side_b: u32 = env::read();

    let hypotenuse = compute_hypotenuse(side_a, side_b);

    env::commit(&hypotenuse);
}

fn compute_hypotenuse(a: u32, b: u32) -> u32 {
    let a_squared = a.pow(2);
    let b_squared = b.pow(2);
    let sum = a_squared + b_squared;
    
    sqrt_u32(sum)
}

fn sqrt_u32(mut number: u32) -> u32 {
    let mut result = 0;
    let mut bit = 1 << 30; // The second-to-top bit is set: 1<<30 for u32

    // "bit" starts at the highest power of four <= the argument.
    while bit > number {
        bit >>= 2;
    }

    while bit != 0 {
        if number >= result + bit {
            number -= result + bit;
            result = (result >> 1) + bit;
        } else {
            result >>= 1;
        }
        bit >>= 2;
    }

    result
}