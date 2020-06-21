// NOTE: ranges (like `0..=x`) should not be used in the algorithms of this library, since they can
// generate references to `memcpy` in unoptimized code. this code is intended to be used by
// `compiler-builtins` which cannot use `memcpy`.

#![feature(unsafe_block_in_unsafe_fn)]
#![deny(unsafe_op_in_unsafe_fn)]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(feature = "asm", feature(asm))]

#[cfg(test)]
extern crate rand;
#[cfg(test)]
use rand::random;

#[macro_use]
mod misc;

#[macro_use]
mod binary_long;

#[macro_use]
mod delegate;

#[macro_use]
mod trifecta;

#[macro_use]
mod asymmetric;

// `leading_zeros` would be placed in `misc.rs`, but has been placed here because we want to
// minimize changes to any of the files outside of `lib.rs` when copying and pasting to
// `compiler-builtins`.

/// Returns the number of leading binary zeros in `x`.
///
/// Counting leading zeros is performance critical to certain division algorithms and more. This is
/// a fast software routine for CPU architectures without an assembly instruction to quickly
/// calculate `leading_zeros`.
pub const fn leading_zeros(x: usize) -> usize {
    // Note: This routine produces the correct value for `x == 0`. Zero is probably common enough
    // that it could warrant adding a zero check at the beginning, but it was decided not to do
    // this. This code is meant to be the routine for `compiler-builtins` functions like `__clzsi2`
    // which have a precondition that `x != 0`. Compilers will insert the check for zero in cases
    // where it is needed.

    // The base idea is to mask the higher bits of `x` and compare them to zero to bisect the number
    // of leading zeros. For an example, if we are finding the leading zeros of a `u8`, we could
    // check if `x & 0b1111_0000` is zero. If it is zero, then the number of leading zeros is at
    // least 4, otherwise it is less than 4. If `(x & 0b1111_0000) == 0`, then we could branch to
    // another check for if `x & 0b1111_1100` is zero. If `(x & 0b1111_1100) != 0`, then the number
    // of leading zeros is at least 4 but less than 6. One more bisection with `x & 0b1111_1000`
    // determines the number of leading zeros to be 4 if `(x & 0b1111_1000) != 0` and 5 otherwise.
    //
    // However, we do not want to have 6 levels of bisection to 64 leaf nodes and hundreds of bytes
    // of instruction code if `usize` has a bit width of 64. It is possible for all branches of the
    // bisection to use the same code path by conditional shifting and focusing on smaller parts:
    /*
    let mut x = x;
    // temporary
    let mut t: usize;
    // The number of potential leading zeros, assuming that `usize` has a bitwidth of 64 bits
    let mut z: usize = 64;

    t = x >> 32;
    if t != 0 {
        // one of the upper 32 bits is set, so the 32 lower bits are now irrelevant and can be
        // removed from the number of potential leading zeros
        z -= 32;
        // shift `x` so that the next step can deal with the upper 32 bits, otherwise the lower 32
        // bits would be checked by the next step.
        x = t;
    }
    t = x >> 16;
    if t != 0 {
        z -= 16;
        x = t;
    }
    t = x >> 8;
    if t != 0 {
        z -= 8;
        x = t;
    }
    t = x >> 4;
    if t != 0 {
        z -= 4;
        x = t;
    }
    t = x >> 2;
    if t != 0 {
        z -= 2;
        x = t;
    }
    // combine the last two steps
    t = x >> 1;
    if t != 0 {
        return z - 2;
    } else {
        return z - x;
    }
    */
    // The above method has short branches in it which can cause pipeline problems on some
    // platforms, or the compiler removes the branches but at the cost of huge instruction counts
    // from heavy bit manipulation. We can remove the branches by turning `(x & constant) != 0`
    // boolean expressions into an integer (performed on many architecture with a set-if-not-equal
    // instruction).

    // Adapted from LLVM's `compiler-rt/lib/builtins/clzsi2.c`. The original sets the number of
    // zeros `z` to be 0 and adds to that:
    //
    // // If the upper bits are zero, set `t` to `1 << level`
    // let t = (((x & const) == 0) as usize) << level;
    // // If the upper bits are zero, the right hand side expression cancels to zero and no shifting
    // // occurs
    // x >>= (1 << level) - t;
    // // If the upper bits are zero, `1 << level` is added to `z`
    // z += t;
    //
    // It saves some instructions to start `z` at 64 and subtract from that with a negated process
    // instead. Architectures with a set-if-not-equal instruction probably also have a
    // set-if-more-than-or-equal instruction, so we use that. If the architecture does not have such
    // and instruction and has to branch, this is still probably the fastest method:
    //
    // // use `>=` comparison instead
    // let t = ((x >= const) as usize) << level;
    // // shift if the upper bits are not zero
    // x >>= t;
    // // subtract from the number of potential leading zeros
    // z -= t;

    let mut x = x;
    // The number of potential leading zeros
    let mut z = {
        #[cfg(target_pointer_width = "64")]
        {
            64
        }
        #[cfg(target_pointer_width = "32")]
        {
            32
        }
        #[cfg(target_pointer_width = "16")]
        {
            16
        }
    };

    // a temporary
    let mut t: usize;

    #[cfg(target_pointer_width = "64")]
    {
        t = ((x >= (1 << 32)) as usize) << 5;
        x >>= t;
        z -= t;
    }

    #[cfg(any(target_pointer_width = "32", target_pointer_width = "64"))]
    {
        t = ((x >= (1 << 16)) as usize) << 4;
        x >>= t;
        z -= t;
    }

    t = ((x >= (1 << 8)) as usize) << 3;
    x >>= t;
    z -= t;

    t = ((x >= (1 << 4)) as usize) << 2;
    x >>= t;
    z -= t;

    t = ((x >= (1 << 2)) as usize) << 1;
    x >>= t;
    z -= t;

    t = (x >= (1 << 1)) as usize;
    x >>= t;
    z -= t;

    // All bits except LSB are guaranteed to be zero by this point. If `x != 0` then `x == 1` and
    // subtracts a potential zero from `z`.
    z - x

    // We could potentially save a few cycles by using the LUT trick from
    // "https://embeddedgurus.com/state-space/2014/09/
    // fast-deterministic-and-portable-counting-leading-zeros/". 256 bytes for a LUT is too large,
    // so we could perform bisection down to `((x >= (1 << 4)) as usize) << 2` and use this 16 byte LUT
    // for the rest of the work.
    //const LUT: [u8; 16] = [0, 1, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 4, 4, 4, 4];
    //z -= LUT[x] as usize;
    //z
    // However, it ends up generating about the same amount of instructions. When benchmarked on
    // x86_64, it is slightly faster to use the LUT, but I think this is only because of OOO
    // execution effects. Changing to using a LUT and branching is risky for smaller cores.
}

#[test]
fn leading_zeros_test() {
    // binary fuzzer
    let mut x = 0usize;
    let mut ones: usize;
    // creates a mask for indexing the bits of the type
    let bit_indexing_mask = usize::MAX.count_ones() - 1;
    for _ in 0..1000 {
        for _ in 0..4 {
            let r0: u32 = bit_indexing_mask & random::<u32>();
            ones = !0 >> r0;
            let r1: u32 = bit_indexing_mask & random::<u32>();
            let mask = ones.rotate_left(r1);
            match (random(), random()) {
                (false, false) => x |= mask,
                (false, true) => x &= mask,
                (true, _) => x ^= mask,
            }
        }
        if leading_zeros(x) != (x.leading_zeros() as usize) {
            panic!(
                "x: {}, expected: {}, found: {}",
                x,
                x.leading_zeros(),
                leading_zeros(x)
            );
        }
    }
}

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

// note: there are some architecture dependent `#[cfg(...)]`s in the macro
impl_normalization_shift!(u8_normalization_shift, 8, u8, i8, inline);
impl_normalization_shift!(u16_normalization_shift, 16, u16, i16, inline);
impl_normalization_shift!(u32_normalization_shift, 32, u32, i32, inline);
impl_normalization_shift!(u64_normalization_shift, 64, u64, i64, inline);

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
    i8_div_rem_binary_long
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
    i16_div_rem_binary_long
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
    i32_div_rem_delegate
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
    i64_div_rem_asymmetric
);

// 128 bit
impl_delegate!(
    u128_div_rem_delegate,
    i128_div_rem_delegate,
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
    i128_div_rem_asymmetric
);
