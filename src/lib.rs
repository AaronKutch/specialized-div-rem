// NOTE: ranges (like `0..=x`) should not be used in the algorithms of this library, since they can
// generate references to `memcpy` in unoptimized code. this code is intended to be used by
// `compiler-builtins` which cannot use `memcpy`.

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(feature = "asm", feature(asm))]

// TODO: when `unsafe_block_in_unsafe_fn` is stabilized, remove this
#![allow(unused_unsafe)]

#[cfg(test)]
extern crate rand;
#[cfg(test)]
use rand::random;

#[macro_use]
mod norm_shift;

#[macro_use]
mod binary_long;

#[macro_use]
mod delegate;

#[macro_use]
mod trifecta;

#[macro_use]
mod asymmetric;

#[macro_use]
mod extra;
pub use extra::usize_leading_zeros;

#[inline]
fn u16_by_u16_div_rem(duo: u16, div: u16) -> (u16, u16) {
    (duo / div, duo % div)
}

#[inline]
fn u32_by_u32_div_rem(duo: u32, div: u32) -> (u32, u32) {
    (duo / div, duo % div)
}

#[cfg(any(not(feature = "asm"), not(target_arch = "x86")))]
#[inline]
unsafe fn u64_by_u32_div_rem(duo: u64, div: u32) -> (u32, u32) {
    let duo_hi = (duo >> 32) as u32;
    debug_assert!(duo_hi < div);
    ((duo / (div as u64)) as u32, (duo % (div as u64)) as u32)
}

/// Divides `duo` by `div` and returns a tuple of the quotient and the remainder.
///
/// # Safety
///
/// If the quotient does not fit in a `u32`, or `div == 0`, a floating point exception happens.
#[cfg(all(feature = "asm", target_arch = "x86"))]
#[inline]
unsafe fn u64_by_u32_div_rem(duo: u64, div: u32) -> (u32, u32) {
    let duo_lo = duo as u32;
    let duo_hi = (duo >> 32) as u32;
    debug_assert!(duo_hi < div);
    let quo: u32;
    let rem: u32;
    unsafe {
        // divides the combined registers rdx:rax (`duo` is split into two 32 bit parts to do this)
        // by `div`. The quotient is stored in rax and the remainder in rdx.
        asm!(
            "div {0}",
            in(reg) div,
            inlateout("rax") duo_lo => quo,
            inlateout("rdx") duo_hi => rem,
            options(pure, nomem, nostack)
        );
    }
    (quo, rem)
}

#[inline]
fn u64_by_u64_div_rem(duo: u64, div: u64) -> (u64, u64) {
    (duo / div, duo % div)
}

#[cfg(any(not(feature = "asm"), not(target_arch = "x86_64")))]
#[inline]
unsafe fn u128_by_u64_div_rem(duo: u128, div: u64) -> (u64, u64) {
    let duo_hi = (duo >> 64) as u64;
    debug_assert!(duo_hi < div);
    ((duo / (div as u128)) as u64, (duo % (div as u128)) as u64)
}

/// Divides `duo` by `div` and returns a tuple of the quotient and the remainder.
///
/// # Safety
///
/// If the quotient does not fit in a `u64`, or `div == 0`, a floating point exception happens.
#[cfg(all(feature = "asm", target_arch = "x86_64"))]
#[inline]
unsafe fn u128_by_u64_div_rem(duo: u128, div: u64) -> (u64, u64) {
    let duo_lo = duo as u64;
    let duo_hi = (duo >> 64) as u64;
    debug_assert!(duo_hi < div);
    let quo: u64;
    let rem: u64;
    unsafe {
        // divides the combined registers rdx:rax (`duo` is split into two 64 bit parts to do this)
        // by `div`. The quotient is stored in rax and the remainder in rdx.
        asm!(
            "div {0}",
            in(reg) div,
            inlateout("rax") duo_lo => quo,
            inlateout("rdx") duo_hi => rem,
            options(pure, nomem, nostack)
        );
    }
    (quo, rem)
}

// TODO: if Rust adds a way to check for the "B" extension on RISC-V, then modify this
#[cfg(any(feature = "no_lz", target_arch = "riscv32", target_arch = "riscv64"))]
const USE_LZ: bool = false;

// The rest of the common architectures supply `leading_zeros`
#[cfg(not(any(feature = "no_lz", target_arch = "riscv32", target_arch = "riscv64")))]
const USE_LZ: bool = true;

impl_normalization_shift!(u8_normalization_shift, USE_LZ, 8, u8, i8, inline);
impl_normalization_shift!(u16_normalization_shift, USE_LZ, 16, u16, i16, inline);
impl_normalization_shift!(u32_normalization_shift, USE_LZ, 32, u32, i32, inline);
impl_normalization_shift!(u64_normalization_shift, USE_LZ, 64, u64, i64, inline);

// Note: one reason for the macros having a `$half_division:ident` instead of directly calling the
// `/` and `%` builtin operators is that allows using different algorithms for the half
// division instead of just the default.
//
// One result of benchmarking is that, when hardware division is not availiable and the u64 divisions
// require a `u32_div_rem_binary_long` half sized division, the fastest algorithm is the
// `u64_div_rem_delegate` algorithm. When the u128 sized divisions in turn use
// `u64_div_rem_delegate` as their half sized division, the fastest algorithm is
// `u128_div_rem_trifecta` (except if the hardware does not have a fast enough multiplier, in which
// case `u128_div_rem_delegate` should be used).

// Note: The overhead of the existing binary long division algorithm setup is high enough that
// faster algorithms for 8 bit and 16 bit divisions probably exist. However, the smallest division
// in `compiler-builtins` is 32 bits, so these cases are only left in for testing purposes.

// 8 bit
impl_binary_long!(
    u8_div_rem_binary_long,
    i8_div_rem_binary_long,
    u8_normalization_shift,
    8,
    u8,
    i8,
    inline;
    inline
);
test!(
    8,
    u8,
    i8,
    div_rem_binary_long_8,
    u8_div_rem_binary_long,
    i8_div_rem_binary_long;
);

// 16 bit
impl_binary_long!(
    u16_div_rem_binary_long,
    i16_div_rem_binary_long,
    u16_normalization_shift,
    16,
    u16,
    i16,
    inline;
    inline
);
test!(
    16,
    u16,
    i16,
    div_rem_binary_long_16,
    u16_div_rem_binary_long,
    i16_div_rem_binary_long;
);

// 32 bit
impl_binary_long!(
    u32_div_rem_binary_long,
    i32_div_rem_binary_long,
    u32_normalization_shift,
    32,
    u32,
    i32,
    inline;
    inline
);
impl_delegate!(
    u32_div_rem_delegate,
    i32_div_rem_delegate,
    u16_normalization_shift,
    u16_by_u16_div_rem,
    8,
    u8,
    u16,
    u32,
    i32,
    inline;
    inline
);
test!(
    32,
    u32,
    i32,
    div_rem_binary_long_32,
    u32_div_rem_binary_long,
    i32_div_rem_binary_long;
    div_rem_delegate_32,
    u32_div_rem_delegate,
    i32_div_rem_delegate;
);

// 64 bit
impl_binary_long!(
    u64_div_rem_binary_long,
    i64_div_rem_binary_long,
    u64_normalization_shift,
    64,
    u64,
    i64,
    inline;
    inline
);
impl_delegate!(
    u64_div_rem_delegate,
    i64_div_rem_delegate,
    u32_normalization_shift,
    u32_by_u32_div_rem,
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
    u32_by_u32_div_rem,
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
    u32_by_u32_div_rem,
    u64_by_u32_div_rem,
    16,
    u16,
    u32,
    u64,
    i64,
    inline;
    inline
);
test!(
    64,
    u64,
    i64,
    div_rem_binary_long_64,
    u64_div_rem_binary_long,
    i64_div_rem_binary_long;
    div_rem_delegate_64,
    u64_div_rem_delegate,
    i64_div_rem_delegate;
    div_rem_trifecta_64,
    u64_div_rem_trifecta,
    i64_div_rem_trifecta;
    div_rem_asymmetric_64,
    u64_div_rem_asymmetric,
    i64_div_rem_asymmetric;
);

// 128 bit
impl_delegate!(
    u128_div_rem_delegate,
    i128_div_rem_delegate,
    u64_normalization_shift,
    u64_by_u64_div_rem,
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
    u64_by_u64_div_rem,
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
    u64_by_u64_div_rem,
    u128_by_u64_div_rem,
    32,
    u32,
    u64,
    u128,
    i128,
    inline;
    inline
);
test!(
    128,
    u128,
    i128,
    div_rem_delegate_128,
    u128_div_rem_delegate,
    i128_div_rem_delegate;
    div_rem_trifecta_128,
    u128_div_rem_trifecta,
    i128_div_rem_trifecta;
    div_rem_asymmetric_128,
    u128_div_rem_asymmetric,
    i128_div_rem_asymmetric;
);
