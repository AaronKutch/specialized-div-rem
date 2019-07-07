//! These division functions use a different type of long division than the binary long division
//! typically used by software to divide integers larger than the size of the CPU hardware
//! division.
#![no_std]
#![cfg_attr(feature = "asm", feature(asm))]

//it would be annoying to convert the test function in the macro to one
//that could be used in a test module
#[cfg(test)]
extern crate rand;

#[macro_use] mod all_all;

/// This function is unsafe, because if the quotient of `duo` and `div` does not fit in a `u64`,
/// a floating point exception is thrown.
#[cfg(all(target_arch = "x86_64", feature = "asm"))]
#[inline]
unsafe fn divrem_128_by_64(duo: u128, div: u64) -> (u64, u64) {
    let quo: u64;
    let rem: u64;
    let duo_lo = duo as u64;
    let duo_hi = (duo >> 64) as u64;
    asm!("divq $4"
        : "={rax}"(quo), "={rdx}"(rem)
        : "{rax}"(duo_lo), "{rdx}"(duo_hi), "r"(div)
        : "rax", "rdx"
    );
    return (quo, rem);
}

// for when the $uD by $uX function cannot be called
#[cfg(any(not(target_arch = "x86_64"), not(feature = "asm")))]
unsafe fn dummy128(duo: u128, div: u64) -> (u64, u64) {(duo as u64, div)}
unsafe fn dummy64(duo: u64, div: u32) -> (u32, u32) {(duo as u32, div)}
unsafe fn dummy32(duo: u32, div: u16) -> (u16, u16) {(duo as u16, div)}

impl_div_rem!(u32_div_rem, i32_div_rem, u32_i32_div_rem_test, 8u32, u8, u16, u32, i32, 0b11111u32, inline; inline; false, dummy32);
impl_div_rem!(u64_div_rem, i64_div_rem, u64_i64_div_rem_test, 16u32, u16, u32, u64, i64, 0b111111u32, inline; inline; false, dummy64);
#[cfg(any(not(target_arch = "x86_64"), not(feature = "asm")))]
impl_div_rem!(u128_div_rem, i128_div_rem, u128_i128_div_rem_test, 32u32, u32, u64, u128, i128, 0b1111111u32, inline; inline; false, dummy128);
#[cfg(all(target_arch = "x86_64", feature = "asm"))]
impl_div_rem!(u128_div_rem, i128_div_rem, u128_i128_div_rem_test, 32u32, u32, u64, u128, i128, 0b1111111u32, inline; inline; true, divrem_128_by_64);
