#![cfg_attr(feature = "no_std", no_std)]
#![cfg_attr(feature = "asm", feature(asm))]

// it would be annoying to convert the test function in the macro to one
// that could be used in a test module
#[cfg(test)]
extern crate rand;

#[macro_use]
mod binary_shift;

#[macro_use]
mod trifecta;

#[macro_use]
mod asymmetric;

/// This function is unsafe, because if the quotient of `duo` and `div` does not
/// fit in a `u64`, a floating point exception is thrown.
#[cfg(all(target_arch = "x86_64", feature = "asm"))]
#[inline]
unsafe fn u128_div_u64(duo: u128, div: u64) -> (u64, u64) {
    let quo: u64;
    let rem: u64;
    let duo_lo = duo as u64;
    let duo_hi = (duo >> 64) as u64;
    asm!("divq $4"
        : "={rax}"(quo), "={rdx}"(rem)
        : "{rax}"(duo_lo), "{rdx}"(duo_hi), "r"(div)
        : "rax", "rdx"
    );
    return (quo, rem)
}

// for when the $uD by $uX assembly function cannot be called
#[cfg(any(not(target_arch = "x86_64"), not(feature = "asm")))]
unsafe fn u128_div_u64(duo: u128, div: u64) -> (u64, u64) {
    ((duo / (div as u128)) as u64, (duo % (div as u128)) as u64)
}
unsafe fn u64_div_u32(duo: u64, div: u32) -> (u32, u32) {
    ((duo / (div as u64)) as u32, (duo % (div as u64)) as u32)
}
unsafe fn u32_div_u16(duo: u32, div: u16) -> (u16, u16) {
    ((duo / (div as u32)) as u16, (duo % (div as u32)) as u16)
}

impl_binary_shift!(
    u32_div_rem_binary_shift,
    i32_div_rem_binary_shift,
    div_rem_binary_shift_32,
    8,
    u8,
    u16,
    u32,
    i32,
    inline;
    inline
);
impl_trifecta!(
    u32_div_rem_trifecta,
    i32_div_rem_trifecta,
    div_rem_trifecta_32,
    8,
    u8,
    u16,
    u32,
    i32,
    inline;
    inline
);
impl_asymmetric!(
    u32_div_rem_asymmetric,
    i32_div_rem_asymmetric,
    div_rem_asymmetric_32,
    u32_div_u16,
    8,
    u8,
    u16,
    u32,
    i32,
    inline;
    inline
);
impl_binary_shift!(
    u64_div_rem_binary_shift,
    i64_div_rem_binary_shift,
    div_rem_binary_shift_64,
    16,
    u16,
    u32,
    u64,
    i64,
    inline;
    inline
);
impl_trifecta!(
    u64_div_rem_trifecta,
    i64_div_rem_trifecta,
    div_rem_trifecta_64,
    16,
    u16,
    u32,
    u64,
    i64,
    inline;
    inline
);
impl_asymmetric!(
    u64_div_rem_asymmetric,
    i64_div_rem_asymmetric,
    div_rem_asymmetric_64,
    u64_div_u32,
    16,
    u16,
    u32,
    u64,
    i64,
    inline;
    inline
);
impl_binary_shift!(
    u128_div_rem_binary_shift,
    i128_div_rem_binary_shift,
    div_rem_binary_shift_128,
    32,
    u32,
    u64,
    u128,
    i128,
    inline;
    inline
);
impl_trifecta!(
    u128_div_rem_trifecta,
    i128_div_rem_trifecta,
    div_rem_trifecta_128,
    32,
    u32,
    u64,
    u128,
    i128,
    inline;
    inline
);
impl_asymmetric!(
    u128_div_rem_asymmetric,
    i128_div_rem_asymmetric,
    div_rem_asymmetric_128,
    u128_div_u64,
    32,
    u32,
    u64,
    u128,
    i128,
    inline;
    inline
);
