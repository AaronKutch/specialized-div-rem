//! These division functions use a different type of long division than the binary long division
//! typically used by software to divide integers larger than the size of the CPU hardware
//! division.
#![no_std]
#![feature(asm)]

//it would be annoying to convert the test function in the macro to one
//that could be used in a test module
#[cfg(test)]
extern crate rand;

mod all_all;

pub use self::all_all::*;