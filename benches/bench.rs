#![feature(test)]
extern crate rand;
extern crate test;
use rand::prelude::*;
use std::{i128, u128, u32, u64};
use test::black_box;
use test::Bencher;

extern crate specialized_div_rem;
use specialized_div_rem::*;

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
fn constant_u128_div_rem_long(bencher: &mut Bencher) {
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
                    let temp0: $ty = random();
                    a.push(temp0);
                    let temp1: $ty = random();
                    if temp1 == $ty_zero {
                        b.push($ty_one);
                    } else {
                        b.push(temp1);
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

//produces two benchmarking functions that iterates a standard rust operation and an alternative operation on an precomputed array of 1024 random integers.The iterator is passed to a `Bencher`, and when `cargo bench` is used, it returns the time to do all 1024 operations. Note that some time is taken to lookup the array values and add the result of the operation, so subtract the appropriate `{}_baseline` values to find a closer value to how long the operations themselves actually take.
macro_rules! std_and_long_bencher {
    (
        $fn:ident, //the special division operation
        $fn_kind:expr, //this is to reduce repeated macro code, it specifies what the operation is
        $ty:tt, //the type that is entered into the operations
        $ty_bits:expr, //the number of bits in a `$ty`
        $ty_zero:expr, //the zero of a `$ty`
        $ty_one:expr, //the one of a `$ty`
        $arg0_sb:expr, //the number of significant random bits in argument 0 to the operation
        $arg1_sb:expr, //the number of significant bits in argument 1 to the operation. Note: argument 1 is set to 1 if the random number generator returns zero
        $name_std:ident, //name of the benchmarking function that uses the standard Rust operations
        $name_long:ident //name of the benchmarking function that uses another operation
    ) => {
        #[bench]
        fn $name_std(bencher: &mut Bencher) {
            let (a,b) = black_box({
                let (mut a,mut b): (Vec<$ty>,Vec<$ty>) = (Vec::new(),Vec::new());
                for _ in 0..32 {
                    let temp0: $ty = random();
                    a.push(temp0 & ($ty::MAX >> ($ty_bits - $arg0_sb)));
                    let temp1: $ty = random();
                    if temp1 == $ty_zero {
                        b.push($ty_one);
                    } else {
                        b.push(temp1 & ($ty::MAX >> ($ty_bits - $arg1_sb)));
                    }
                }
                (a,b)
            });
            bencher.iter(|| {
                let mut s0 = 0;
                let mut s1 = 0;
                match $fn_kind {
                    FnKind::DivRem => for i in 0..a.len() {s0 += a[i] / b[i]; s1 += a[i] % b[i];},
                    FnKind::Div => for i in 0..a.len() {s0 += a[i] / b[i];},
                    FnKind::Rem => for i in 0..a.len() {s1 += a[i] % b[i];},
                }
                (s0, s1)
            })
        }

        #[bench]
        fn $name_long(bencher: &mut Bencher) {
            let (a,b) = black_box({
                let (mut a,mut b): (Vec<$ty>,Vec<$ty>) = (Vec::new(),Vec::new());
                for _ in 0..32 {
                    let temp0: $ty = random();
                    a.push(temp0 & ($ty::MAX >> ($ty_bits - $arg0_sb)));
                    let temp1: $ty = random();
                    if temp1 == $ty_zero {
                        b.push($ty_one);
                    } else {
                        b.push(temp1 & ($ty::MAX >> ($ty_bits - $arg1_sb)));
                    }
                }
                (a,b)
            });
            bencher.iter(|| {
                let mut s0 = 0;
                let mut s1 = 0;
                match $fn_kind {
                    FnKind::DivRem => for i in 0..a.len() {let temp = $fn(a[i],b[i]); s0 += temp.0; s1 += temp.1;},
                    FnKind::Div => for i in 0..a.len() {s0 += $fn(a[i],b[i]).0;},
                    FnKind::Rem => for i in 0..a.len() {s1 += $fn(a[i],b[i]).1;},
                }
                (s0, s1)
            })
        }
    };
}

std_and_long_bencher!(
    u32_div_rem,
    FnKind::DivRem,
    u32,
    32,
    0u32,
    1u32,
    32,
    32 - 8,
    u32_div_rem_all_mid_std,
    u32_div_rem_all_mid_long
);
std_and_long_bencher!(
    u64_div_rem,
    FnKind::DivRem,
    u64,
    64,
    0u64,
    1u64,
    64,
    64 - 16,
    u64_div_rem_all_mid_std,
    u64_div_rem_all_mid_long
);
std_and_long_bencher!(
    u128_div_rem,
    FnKind::DivRem,
    u128,
    128,
    0u128,
    1u128,
    128,
    128 - 32,
    u128_div_rem_all_mid_std,
    u128_div_rem_all_mid_long
);
std_and_long_bencher!(
    i128_div_rem,
    FnKind::DivRem,
    i128,
    128,
    0i128,
    1i128,
    128,
    128 - 32,
    i128_div_rem_all_mid_std,
    i128_div_rem_all_mid_long
);

std_and_long_bencher!(
    u32_div_rem,
    FnKind::Div,
    u32,
    32,
    0u32,
    1u32,
    32,
    32 - 8,
    u32_div_all_mid_std,
    u32_div_all_mid_long
);
std_and_long_bencher!(
    u64_div_rem,
    FnKind::Div,
    u64,
    64,
    0u64,
    1u64,
    64,
    64 - 16,
    u64_div_all_mid_std,
    u64_div_all_mid_long
);
std_and_long_bencher!(
    u128_div_rem,
    FnKind::Div,
    u128,
    128,
    0u128,
    1u128,
    128,
    128 - 32,
    u128_div_all_mid_std,
    u128_div_all_mid_long
);

std_and_long_bencher!(
    u128_div_rem,
    FnKind::Rem,
    u128,
    128,
    0u128,
    1u128,
    128,
    128 - 32,
    u128_rem_all_mid_std,
    u128_rem_all_mid_long
);

std_and_long_bencher!(
    u128_div_rem,
    FnKind::DivRem,
    u128,
    128,
    0u128,
    1u128,
    128,
    128,
    u128_div_rem_all_all_std,
    u128_div_rem_all_all_long
);
std_and_long_bencher!(
    u128_div_rem,
    FnKind::DivRem,
    u128,
    128,
    0u128,
    1u128,
    128,
    32,
    u128_div_rem_all_0_std,
    u128_div_rem_all_0_long
);
std_and_long_bencher!(
    u128_div_rem,
    FnKind::Div,
    u128,
    128,
    0u128,
    1u128,
    128,
    128,
    u128_div_all_all_std,
    u128_div_all_all_long
);
std_and_long_bencher!(
    u128_div_rem,
    FnKind::Div,
    u128,
    128,
    0u128,
    1u128,
    128,
    32,
    u128_div_all_0_std,
    u128_div_all_0_long
);
