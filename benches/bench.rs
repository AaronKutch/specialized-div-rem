#![feature(test)]
#![cfg_attr(feature = "asm", feature(asm))]
extern crate test;
use rand::prelude::*;
use std::{i128, u128, u32, u64};
use test::black_box;
use test::Bencher;

extern crate specialized_div_rem;
use specialized_div_rem::*;

/*
// test for u128_conquer
macro_rules! impl_div_rem {
    (
        $unsigned_name:ident, //name of the unsigned function
        $test_name:ident, //name of the test function
        $n_h:expr, //the number of bits in $iH or $uH
        $uH:ident, //unsigned integer with half the bit width of $uX
        $uX:ident, //the largest division instruction that this function calls operates on this
        $uD:ident, //unsigned integer with double the bit width of $uX
        $bit_selector_max:expr //the max value of the smallest bit string needed to index the bits of an $uD
    ) => {
        #[test]
        fn $test_name() {
            type T = $uD;
            let n = $n_h * 4;
            // checks all possible single continuous strings of ones (except when all bits are zero)
            // uses about 68 million iterations for T = u128
            let mut lhs0: T = 1;
            for i0 in 1..=n {
                let mut lhs1 = lhs0;
                for i1 in 0..i0 {
                    let mut rhs0: T = 1;
                    for i2 in 1..=n {
                        let mut rhs1 = rhs0;
                        for i3 in 0..i2 {
                            assert_eq!(
                                (lhs1.wrapping_div(rhs1),
                                lhs1.wrapping_rem(rhs1)),$unsigned_name(lhs1,rhs1)
                            );
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
            use rand::random;
            let mut lhs: T = 0;
            let mut rhs: T = 0;
            let mut ones: T;
            for _ in 0..10_000_000 {
                let r0: u32 = $bit_selector_max & random::<u32>();
                ones = 0;
                for _ in 0..r0 {
                    ones <<= 1;
                    ones |= 1;
                }
                let r1: u32 = $bit_selector_max & random::<u32>();
                let mask = ones.rotate_left(r1);
                match (random(),random(),random()) {
                    (false,false,false) => lhs |= mask,
                    (false,false,true) => lhs &= mask,
                    (false,true,false) => lhs ^= mask,
                    (false,true,true) => lhs ^= mask,
                    (true,false,false) => rhs |= mask,
                    (true,false,true) => rhs &= mask,
                    (true,true,false) => rhs ^= mask,
                    (true,true,true) => rhs ^= mask,
                }
                if rhs != 0 {
                    assert_eq!(
                        (lhs.wrapping_div(rhs), lhs.wrapping_rem(rhs)),
                        $unsigned_name(lhs,rhs)
                    )
                }
            }
        }
    }
}

impl_div_rem!(u128_conquer, u128_conquer_test, 32u32, u32, u64, u128, 0b1111111u32);
*/

/// This function is unsafe, because if the quotient of `duo` and `div` does not fit in a
/// `u64`, a floating point exception is thrown.
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

#[cfg(any(not(target_arch = "x86_64"), not(feature = "asm")))]
#[inline]
unsafe fn divrem_128_by_64(duo: u128, div: u64) -> (u64, u64) {
    return ((duo / (div as u128)) as u64, (duo % (div as u128)) as u64);
}

fn u128_div_u64(duo: u128, div: u128) -> (u128, u128) {
    //assert!(div.leading_zeros() >= 64);
    //assert!((duo / div).leading_zeros() >= 64);
    let (quo, rem) = unsafe { divrem_128_by_64(duo, div as u64) };
    return (quo as u128, rem as u128);
}

// This is a classic divide-and-conquer version different from the binary long division
// used by std and my no-oversubtract division.
// This has been adapted from
// https://www.codeproject.com/tips/785014/uint-division-modulus
fn u128_conquer(duo: u128, div: u128) -> (u128, u128) {
    let duo_lo = duo as u64;
    let duo_hi = (duo >> 64) as u64;
    let div_lo = div as u64;
    let div_hi = (div >> 64) as u64;
    if div_hi == 0 {
        if div_lo == 0 {
            panic!();
        }
        if duo_hi < div_lo {
            // plain u128 by u64 division that will fit into u64
            return u128_div_u64(duo, div);
        } else {
            let (quo_hi, tmp) = (duo_hi / div_lo, duo_hi % div_lo);
            let quo_rem = u128_div_u64(
                (duo_lo as u128) | ((tmp as u128) << 64),
                div_lo as u128
            );
            return (
                (quo_rem.0 as u128) | ((quo_hi as u128) << 64),
                quo_rem.1
            );
        }
    } else {
        let div_lz = div_hi.leading_zeros();
        let mut quo = u128_div_u64(duo >> 1, (div << div_lz) >> 64).0 >> (63 - div_lz);

        if quo != 0 {
            quo -= 1;
        }
        let mut rem = duo - (quo * div);

        if rem >= div {
            quo += 1;
            rem -= div;
        }
        (quo, rem)
    }
}

const C: [u128; 6] = [
    128756765776577777777777778989897568763u128,
    4586787978568756634253423453454363u128,
    45856875663425342345345436u128,
    45867873423453454363u128,
    867834545436u128,
    1786u128,
];

#[bench]
fn constant_u128_div_rem_std(bencher: &mut Bencher) {
    let a = black_box(C);
    bencher.iter(|| {
        let mut sum0 = 0u128;
        let mut sum1 = 0u128;
        for i0 in 0..a.iter().len() {
            for i1 in 0..a.iter().len() {
                sum0 += a[i0] / a[i1];
                sum1 += a[i0] % a[i1];
            }
        }
        (sum0, sum1)
    });
}

#[bench]
fn constant_u128_div_rem_conquer(bencher: &mut Bencher) {
    let a = black_box(C);
    bencher.iter(|| {
        let mut sum0 = 0u128;
        let mut sum1 = 0u128;
        for i0 in 0..a.iter().len() {
            for i1 in 0..a.iter().len() {
                sum0 += u128_conquer(a[i0], a[i1]).0;
                sum1 += u128_conquer(a[i0], a[i1]).1;
            }
        }
        (sum0, sum1)
    });
}

#[bench]
fn constant_u128_div_rem_new(bencher: &mut Bencher) {
    let a = black_box(C);
    bencher.iter(|| {
        let mut sum0 = 0u128;
        let mut sum1 = 0u128;
        for i0 in 0..a.iter().len() {
            for i1 in 0..a.iter().len() {
                sum0 += u128_div_rem(a[i0], a[i1]).0;
                sum1 += u128_div_rem(a[i0], a[i1]).1;
            }
        }
        (sum0, sum1)
    });
}

//makes a function that approximates how long of the `std_and_long_bencher` benchmarks take outside of the actual division and remainder operations
macro_rules! baseline_bencher {
    ($name:ident, $ty:ty, $ty_zero:expr, $ty_one:expr) => {
        #[bench]
        fn $name(bencher: &mut Bencher) {
            let (a, b) = black_box({
                let (mut a, mut b): (Vec<$ty>, Vec<$ty>) = (Vec::new(), Vec::new());
                for _ in 0..32 {
                    let tmp0: $ty = random();
                    a.push(tmp0);
                    let tmp1: $ty = random();
                    if tmp1 == $ty_zero {
                        b.push($ty_one);
                    } else {
                        b.push(tmp1);
                    }
                }
                (a, b)
            });
            bencher.iter(|| {
                let mut s0 = 0;
                let mut s1 = 0;
                for i in 0..a.len() {
                    s0 += a[i].wrapping_add(b[i]);
                    s1 += a[i].wrapping_sub(b[i]);
                }
                (s0, s1)
            })
        }
    };
}

baseline_bencher!(u32_baseline, u32, 0u32, 1u32);
baseline_bencher!(u64_baseline, u64, 0u64, 1u64);
baseline_bencher!(u128_baseline, u128, 0u128, 1u128);

enum FnKind {
    DivRem,
    Div,
    Rem,
}

// The following macros compare three benchmarking functions that use the standard Rust division (`_std`), the classical divide-and-conquer operation (`_conquer`), and my no-oversubtraction division (`_new`). 32 random integers are used for every run. Note that some
// time is taken to lookup the array values and add the result of the operation, so subtract the
// appropriate `{}_baseline` values to find a closer value to how long the operations themselves
// actually take.

macro_rules! std_and_new_bencher {
    (
        $fn_new:ident, // the no-oversubtraction division function
        $fn_kind:expr, //this is to reduce repeated macro code, it specifies what the operation is
        $ty:tt, //the type that is entered into the operations
        $ty_bits:expr, //the number of bits in a `$ty`
        $ty_zero:expr, //the zero of a `$ty`
        $ty_one:expr, //the one of a `$ty`
        $arg0_sb:expr, //the number of significant random bits in argument 0 to the operation
        $arg1_sb:expr, //the number of significant bits in argument 1 to the operation. Note: argument 1 is set to 1 if the random number generator returns zero
        $name_std:ident, //name of the benchmarking function that uses the standard Rust division
        $name_new:ident //name of the benchmarking function that uses the no-oversubtraction division
    ) => {
        #[bench]
        fn $name_std(bencher: &mut Bencher) {
            let (a, b) = black_box({
                let (mut a, mut b): (Vec<$ty>, Vec<$ty>) = (Vec::new(), Vec::new());
                for _ in 0..32 {
                    let tmp0: $ty = random();
                    a.push(tmp0 & ($ty::MAX >> ($ty_bits - $arg0_sb)));
                    let tmp1: $ty = random();
                    if tmp1 == $ty_zero {
                        b.push($ty_one);
                    } else {
                        b.push(tmp1 & ($ty::MAX >> ($ty_bits - $arg1_sb)));
                    }
                }
                (a, b)
            });
            bencher.iter(|| {
                let mut s0 = 0;
                let mut s1 = 0;
                match $fn_kind {
                    FnKind::DivRem => {
                        for i in 0..a.len() {
                            s0 += a[i] / b[i];
                            s1 += a[i] % b[i];
                        }
                    }
                    FnKind::Div => {
                        for i in 0..a.len() {
                            s0 += a[i] / b[i];
                        }
                    }
                    FnKind::Rem => {
                        for i in 0..a.len() {
                            s1 += a[i] % b[i];
                        }
                    }
                }
                (s0, s1)
            })
        }

        #[bench]
        fn $name_new(bencher: &mut Bencher) {
            let (a, b) = black_box({
                let (mut a, mut b): (Vec<$ty>, Vec<$ty>) = (Vec::new(), Vec::new());
                for _ in 0..32 {
                    let tmp0: $ty = random();
                    a.push(tmp0 & ($ty::MAX >> ($ty_bits - $arg0_sb)));
                    let tmp1: $ty = random();
                    if tmp1 == $ty_zero {
                        b.push($ty_one);
                    } else {
                        b.push(tmp1 & ($ty::MAX >> ($ty_bits - $arg1_sb)));
                    }
                }
                (a, b)
            });
            bencher.iter(|| {
                let mut s0 = 0;
                let mut s1 = 0;
                match $fn_kind {
                    FnKind::DivRem => {
                        for i in 0..a.len() {
                            let tmp = $fn_new(a[i], b[i]);
                            s0 += tmp.0;
                            s1 += tmp.1;
                        }
                    }
                    FnKind::Div => {
                        for i in 0..a.len() {
                            s0 += $fn_new(a[i], b[i]).0;
                        }
                    }
                    FnKind::Rem => {
                        for i in 0..a.len() {
                            s1 += $fn_new(a[i], b[i]).1;
                        }
                    }
                }
                (s0, s1)
            })
        }
    };
}

macro_rules! std_conquer_new_bencher {
    (
        $fn_new:ident, // the no-oversubtraction division function
        $fn_conquer:ident, // the divide-and-conquer division function
        $fn_kind:expr, //this is to reduce repeated macro code, it specifies what the operation is
        $ty:tt, //the type that is entered into the operations
        $ty_bits:expr, //the number of bits in a `$ty`
        $ty_zero:expr, //the zero of a `$ty`
        $ty_one:expr, //the one of a `$ty`
        $arg0_sb:expr, //the number of significant random bits in argument 0 to the operation
        $arg1_sb:expr, //the number of significant bits in argument 1 to the operation. Note: argument 1 is set to 1 if the random number generator returns zero
        $name_std:ident, //name of the benchmarking function that uses the standard Rust division
        $name_conquer:ident, //name of the benchmarking function that uses the classic divide-and-conquer division
        $name_new:ident //name of the benchmarking function that uses the no-oversubtraction division
    ) => {
        #[bench]
        fn $name_std(bencher: &mut Bencher) {
            let (a, b) = black_box({
                let (mut a, mut b): (Vec<$ty>, Vec<$ty>) = (Vec::new(), Vec::new());
                for _ in 0..32 {
                    let tmp0: $ty = random();
                    a.push(tmp0 & ($ty::MAX >> ($ty_bits - $arg0_sb)));
                    let tmp1: $ty = random();
                    if tmp1 == $ty_zero {
                        b.push($ty_one);
                    } else {
                        b.push(tmp1 & ($ty::MAX >> ($ty_bits - $arg1_sb)));
                    }
                }
                (a, b)
            });
            bencher.iter(|| {
                let mut s0 = 0;
                let mut s1 = 0;
                match $fn_kind {
                    FnKind::DivRem => {
                        for i in 0..a.len() {
                            s0 += a[i] / b[i];
                            s1 += a[i] % b[i];
                        }
                    }
                    FnKind::Div => {
                        for i in 0..a.len() {
                            s0 += a[i] / b[i];
                        }
                    }
                    FnKind::Rem => {
                        for i in 0..a.len() {
                            s1 += a[i] % b[i];
                        }
                    }
                }
                (s0, s1)
            })
        }

        #[bench]
        fn $name_conquer(bencher: &mut Bencher) {
            let (a, b) = black_box({
                let (mut a, mut b): (Vec<$ty>, Vec<$ty>) = (Vec::new(), Vec::new());
                for _ in 0..32 {
                    let tmp0: $ty = random();
                    a.push(tmp0 & ($ty::MAX >> ($ty_bits - $arg0_sb)));
                    let tmp1: $ty = random();
                    if tmp1 == $ty_zero {
                        b.push($ty_one);
                    } else {
                        b.push(tmp1 & ($ty::MAX >> ($ty_bits - $arg1_sb)));
                    }
                }
                (a, b)
            });
            bencher.iter(|| {
                let mut s0 = 0;
                let mut s1 = 0;
                match $fn_kind {
                    FnKind::DivRem => {
                        for i in 0..a.len() {
                            let tmp = $fn_conquer(a[i], b[i]);
                            s0 += tmp.0;
                            s1 += tmp.1;
                        }
                    }
                    FnKind::Div => {
                        for i in 0..a.len() {
                            s0 += $fn_conquer(a[i], b[i]).0;
                        }
                    }
                    FnKind::Rem => {
                        for i in 0..a.len() {
                            s1 += $fn_conquer(a[i], b[i]).1;
                        }
                    }
                }
                (s0, s1)
            })
        }

        #[bench]
        fn $name_new(bencher: &mut Bencher) {
            let (a, b) = black_box({
                let (mut a, mut b): (Vec<$ty>, Vec<$ty>) = (Vec::new(), Vec::new());
                for _ in 0..32 {
                    let tmp0: $ty = random();
                    a.push(tmp0 & ($ty::MAX >> ($ty_bits - $arg0_sb)));
                    let tmp1: $ty = random();
                    if tmp1 == $ty_zero {
                        b.push($ty_one);
                    } else {
                        b.push(tmp1 & ($ty::MAX >> ($ty_bits - $arg1_sb)));
                    }
                }
                (a, b)
            });
            bencher.iter(|| {
                let mut s0 = 0;
                let mut s1 = 0;
                match $fn_kind {
                    FnKind::DivRem => {
                        for i in 0..a.len() {
                            let tmp = $fn_new(a[i], b[i]);
                            s0 += tmp.0;
                            s1 += tmp.1;
                        }
                    }
                    FnKind::Div => {
                        for i in 0..a.len() {
                            s0 += $fn_new(a[i], b[i]).0;
                        }
                    }
                    FnKind::Rem => {
                        for i in 0..a.len() {
                            s1 += $fn_new(a[i], b[i]).1;
                        }
                    }
                }
                (s0, s1)
            })
        }
    };
}

std_and_new_bencher!(
    u32_div_rem,
    FnKind::DivRem,
    u32,
    32,
    0u32,
    1u32,
    32,
    24,
    u32_div_rem_32_24_std,
    u32_div_rem_32_24_new
);
std_and_new_bencher!(
    u32_div_rem,
    FnKind::DivRem,
    u32,
    32,
    0u32,
    1u32,
    30,
    16,
    u32_div_rem_30_16_std,
    u32_div_rem_30_16_new
);
std_and_new_bencher!(
    u64_div_rem,
    FnKind::DivRem,
    u64,
    64,
    0u64,
    1u64,
    64,
    48,
    u64_div_rem_64_48_std,
    u64_div_rem_64_48_new
);
std_and_new_bencher!(
    u64_div_rem,
    FnKind::DivRem,
    u64,
    64,
    0u64,
    1u64,
    62,
    32,
    u64_div_rem_62_32_std,
    u64_div_rem_62_32_new
);
// see what overhead signed divisions add
std_and_new_bencher!(
    i128_div_rem,
    FnKind::DivRem,
    i128,
    128,
    0i128,
    1i128,
    128,
    96,
    i128_div_rem_128_96_std,
    i128_div_rem_128_96_new
);

std_conquer_new_bencher!(
    u128_div_rem,
    u128_conquer,
    FnKind::DivRem,
    u128,
    128,
    0u128,
    1u128,
    128,
    96,
    u128_div_rem_128_96_std,
    u128_div_rem_128_96_conquer,
    u128_div_rem_128_96_new
);
std_conquer_new_bencher!(
    u128_div_rem,
    u128_conquer,
    FnKind::DivRem,
    u128,
    128,
    0u128,
    1u128,
    126,
    64,
    u128_div_rem_126_64_std,
    u128_div_rem_126_64_conquer,
    u128_div_rem_126_64_new
);
std_conquer_new_bencher!(
    u128_div_rem,
    u128_conquer,
    FnKind::Div,
    u128,
    128,
    0u128,
    1u128,
    128,
    96,
    u128_div_128_96_std,
    u128_div_128_96_conquer,
    u128_div_128_96_new
);
std_conquer_new_bencher!(
    u128_div_rem,
    u128_conquer,
    FnKind::Rem,
    u128,
    128,
    0u128,
    1u128,
    128,
    96,
    u128_rem_128_96_std,
    u128_rem_128_96_conquer,
    u128_rem_128_96_new
);
std_conquer_new_bencher!(
    u128_div_rem,
    u128_conquer,
    FnKind::DivRem,
    u128,
    128,
    0u128,
    1u128,
    128,
    128,
    u128_div_rem_128_128_std,
    u128_div_rem_128_128_conquer,
    u128_div_rem_128_128_new
);
std_conquer_new_bencher!(
    u128_div_rem,
    u128_conquer,
    FnKind::DivRem,
    u128,
    128,
    0u128,
    1u128,
    128,
    32,
    u128_div_rem_128_32_std,
    u128_div_rem_128_32_conquer,
    u128_div_rem_128_32_new
);
std_conquer_new_bencher!(
    u128_div_rem,
    u128_conquer,
    FnKind::DivRem,
    u128,
    128,
    0u128,
    1u128,
    128,
    64,
    u128_div_rem_128_64_std,
    u128_div_rem_128_64_conquer,
    u128_div_rem_128_64_new
);
