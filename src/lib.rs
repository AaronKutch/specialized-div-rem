// NOTE: ranges (like `0..=x`) should not be used in the algorithms of this library, since they can
// generate references to `memcpy` in unoptimized code. this code is intended to be used by
// `compiler-builtins` which cannot use `memcpy`.

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(feature = "asm", feature(asm))]

#[cfg(test)]
extern crate rand;
#[cfg(test)]
use rand::random;

#[macro_use]
mod binary_long;

#[macro_use]
mod carry_left;

#[macro_use]
mod delegate;

#[macro_use]
mod trifecta;

#[macro_use]
mod asymmetric;

/// Returns the number of leading binary zeros in `x`.
///
/// Counting leading zeros is performance critical to certain division algorithms and more. This is
/// a fast routine for CPU architectures without an assembly instruction to quickly calculate
/// `leading_zeros`.
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
    // boolean expressions into an integer.

    // Adapted from LLVM's `compiler-rt/lib/builtins/clzsi2.c`. The original sets the number of
    // zeros `z` to be 0 and add to that:
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
    // instead:
    //
    // // use `!=` comparison instead
    // let t = (((x & const) != 0) as usize) << level;
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
        t = (((x & 0xFFFF_FFFF_0000_0000) != 0) as usize) << 5;
        x >>= t;
        z -= t;
    }

    #[cfg(any(target_pointer_width = "32", target_pointer_width = "64"))]
    {     
        t = (((x & 0xFFFF_0000) != 0) as usize) << 4;
        x >>= t;
        z -= t;
    }

    t = (((x & 0xFF00) != 0) as usize) << 3;
    x >>= t;
    z -= t;

    t = (((x & 0xF0) != 0) as usize) << 2;
    x >>= t;
    z -= t;

    t = (((x & 0b1100) != 0) as usize) << 1;
    x >>= t;
    z -= t;

    t = ((x & 0b10) != 0) as usize;
    x >>= t;
    z -= t;

    // All bits except LSB are guaranteed to be zero by this point. If `x != 0` then `x == 1` and
    // subtracts a potential zero from `z`.
    z - x
}

#[test]
fn lkj() {
    println!("binary_long");
    let duo = 255;
    let div = 3;
    u8_div_rem_binary_long(duo, div);
    println!("carry_left");
    panic!("{:?}", u8_div_rem_carry_left(duo, div));
}

#[test]
fn leading_zeros_test() {
    // binary fuzzer
    let mut x = 0usize;
    let mut ones: usize;
    // creates a mask for indexing the bits of the type
    let bit_selector_max = usize::MAX.count_ones() - 1;
    for _ in 0..1000 {
        for _ in 0..4 {
            let r0: u32 = bit_selector_max & random::<u32>();
            ones = !0 >> r0;
            let r1: u32 = bit_selector_max & random::<u32>();
            let mask = ones.rotate_left(r1);
            match (random(),random()) {
                (false,false) => x |= mask,
                (false,true) => x &= mask,
                (true,_) => x ^= mask,
            }
        }
        if leading_zeros(x) != (x.leading_zeros() as usize) {
            println!("x: {}, expected: {}, found: {}", x, x.leading_zeros(), leading_zeros(x));
        }
        panic!("failed leading_zeros_test");
    }
}

#[inline]
fn u16_by_u16_div_rem(duo: u16, div: u16) -> (u16, u16) {
    (duo / div, duo % div)
}

#[inline]
unsafe fn u32_by_u16_div_rem(duo: u32, div: u16) -> (u16, u16) {
    ((duo / (div as u32)) as u16, (duo % (div as u32)) as u16)
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
    asm!(
        // divides the combined registers rdx:rax (`duo` is split into two 32 bit parts to do this)
        // by `div`. The quotient is stored in rax and the remainder in rdx.
        "div {0}",
        in(reg) div,
        inlateout("rax") duo_lo => quo,
        inlateout("rdx") duo_hi => rem,
        options(pure, nomem, nostack)
    );
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
    asm!(
        // divides the combined registers rdx:rax (`duo` is split into two 64 bit parts to do this)
        // by `div`. The quotient is stored in rax and the remainder in rdx.
        "div {0}",
        in(reg) div,
        inlateout("rax") duo_lo => quo,
        inlateout("rdx") duo_hi => rem,
        options(pure, nomem, nostack)
    );
    (quo, rem)
}

macro_rules! test {
    (
        $n:expr, // the number of bits in a $iX or $uX
        $uX:ident, // unsigned integer that will be shifted
        $iX:ident, // signed version of $uX
        // list of triples of the test name, the unsigned division function, and the signed
        // division function
        $($test_name:ident, $unsigned_name:ident, $signed_name:ident);+
    ) => {
        $(
            #[test]
            fn $test_name() {
                // checks all possible single continuous strings of ones (except when all bits
                // are zero) uses about 68 million iterations for T = u128
                let mut lhs0: $uX = 1;
                for i0 in 1..=$n {
                    let mut lhs1 = lhs0;
                    for i1 in 0..i0 {
                        let mut rhs0: $uX = 1;
                        for i2 in 1..=$n {
                            let mut rhs1 = rhs0;
                            for i3 in 0..i2 {
                                if $unsigned_name(lhs1,rhs1) !=
                                    (
                                        lhs1.wrapping_div(rhs1),
                                        lhs1.wrapping_rem(rhs1)
                                    ) {
                                    println!(
                                        "lhs:{} rhs:{} expected:({}, {}) found:({},{})",
                                        lhs1,
                                        rhs1,
                                        lhs1.wrapping_div(rhs1),
                                        lhs1.wrapping_rem(rhs1),
                                        $unsigned_name(lhs1,rhs1).0,
                                        $unsigned_name(lhs1,rhs1).1
                                    );
                                    panic!("failed division test");
                                }
                                if $signed_name(lhs1 as $iX,rhs1 as $iX) !=
                                    (
                                        (lhs1 as $iX).wrapping_div(rhs1 as $iX),
                                        (lhs1 as $iX).wrapping_rem(rhs1 as $iX)
                                    ) {
                                    println!(
                                        "lhs:{} rhs:{} expected:({}, {}) found:({},{})",
                                        lhs1,
                                        rhs1,
                                        lhs1.wrapping_div(rhs1),
                                        lhs1.wrapping_rem(rhs1),
                                        $signed_name(lhs1 as $iX,rhs1 as $iX).0,
                                        $signed_name(lhs1 as $iX,rhs1 as $iX).1
                                    );
                                    panic!("failed division test");
                                }

                                rhs1 ^= 1 << i3;
                            }
                            rhs0 <<= 1;
                            rhs0 |= 1;
                        }
                        lhs1 ^= 1 << i1;
                    }
                    lhs0 <<= 1;
                    lhs0 |= 1;
                }
                // binary fuzzer
                let mut lhs: $uX = 0;
                let mut rhs: $uX = 0;
                let mut ones: $uX;
                // creates a mask for indexing the bits of the type
                let bit_selector_max = $n - 1;
                for _ in 0..10_000_000 {
                    for _ in 0..4 {
                        let r0: u32 = bit_selector_max & random::<u32>();
                        ones = !0 >> r0;
                        let r1: u32 = bit_selector_max & random::<u32>();
                        let mask = ones.rotate_left(r1);
                        match (random(),random(),random()) {
                            (false,false,false) => lhs |= mask,
                            (false,false,true) => lhs &= mask,
                            (false,true,_) => lhs ^= mask,
                            (true,false,false) => rhs |= mask,
                            (true,false,true) => rhs &= mask,
                            (true,true,_) => rhs ^= mask,
                        }
                    }
                    if rhs != 0 {
                        assert_eq!(
                            (lhs.wrapping_div(rhs), lhs.wrapping_rem(rhs)),
                            $unsigned_name(lhs,rhs)
                        );
                    }
                }
            }
        )+
    }
}

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

// 8 bit
impl_binary_long!(
    u8_div_rem_binary_long,
    i8_div_rem_binary_long,
    8,
    u8,
    i8,
    inline;
    inline
);
impl_carry_left!(
    u8_div_rem_carry_left,
    i8_div_rem_carry_left,
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
    div_rem_carry_left_8,
    u8_div_rem_carry_left,
    i8_div_rem_carry_left
);

// 16 bit
impl_binary_long!(
    u16_div_rem_binary_long,
    i16_div_rem_binary_long,
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
impl_asymmetric!(
    u32_div_rem_asymmetric,
    i32_div_rem_asymmetric,
    u16_by_u16_div_rem,
    u32_by_u16_div_rem,
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
    div_rem_asymmetric_32,
    u32_div_rem_asymmetric,
    i32_div_rem_asymmetric
);

// 64 bit
impl_binary_long!(
    u64_div_rem_binary_long,
    i64_div_rem_binary_long,
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
