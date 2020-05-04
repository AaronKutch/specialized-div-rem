#![feature(test)]
#![cfg_attr(feature = "asm", feature(llvm_asm))]
extern crate test;
use rand::prelude::*;
use std::{i128, u128, u32, u64};
use test::{black_box, Bencher};

extern crate specialized_div_rem;
use specialized_div_rem::*;

pub fn u32_div_rem_std(duo: u32, div: u32) -> (u32, u32) {
    (duo / div, duo % div)
}

pub fn u64_div_rem_std(duo: u64, div: u64) -> (u64, u64) {
    (duo / div, duo % div)
}

pub fn u128_div_rem_std(duo: u128, div: u128) -> (u128, u128) {
    (duo / div, duo % div)
}

pub fn i128_div_rem_std(duo: i128, div: i128) -> (i128, i128) {
    (duo / div, duo % div)
}

enum FnKind {
    DivRem,
    Div,
    Rem,
}

/// This macro can create multiple benchmarking functions that run 32 random integers through a
/// division function. Two masks are applied to `duo` and `div` for testing different ranges of
/// integers.
macro_rules! bencher {
    (
        $fn_kind:expr, // kind of operation
        $ty:tt, // the type that is entered into the operations
        // the size of the mask that is applied to a random number to make the dividend
        $arg0_sb:expr,
        // Note: argument 1 is set to 1 if the random number generator returns zero
        // the size of the mask that is applied to a random number to make the divisor
        $arg1_sb:expr,
        // name of division function and corresponding test
        $($fn_div_rem:ident, $test_name:ident);+;
    ) => {
        $(
            #[bench]
            fn $test_name(bencher: &mut Bencher) {
                let (a, b) = black_box({
                    let bits = std::$ty::MAX.count_ones();
                    let (mut a, mut b): (Vec<$ty>, Vec<$ty>) = (Vec::new(), Vec::new());
                    for _ in 0..32 {
                        let tmp0: $ty = random();
                        a.push(tmp0 & ($ty::MAX >> (bits - $arg0_sb)));
                        let tmp1: $ty = random();
                        let tmp1 = tmp1 & ($ty::MAX >> (bits - $arg1_sb));
                        if tmp1 == 0 {
                            // avoid division by zero
                            b.push(1);
                        } else {
                            b.push(tmp1);
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
                                let tmp = $fn_div_rem(a[i], b[i]);
                                s0 += tmp.0;
                                s1 += tmp.1;
                            }
                        }
                        FnKind::Div => {
                            for i in 0..a.len() {
                                s0 += $fn_div_rem(a[i], b[i]).0;
                            }
                        }
                        FnKind::Rem => {
                            for i in 0..a.len() {
                                s1 += $fn_div_rem(a[i], b[i]).1;
                            }
                        }
                    }
                    (s0, s1)
                })
            }
        )+
    };
}

// These simulate the most common cases
bencher!(
    FnKind::DivRem,
    u32,
    24,
    20,
    u32_div_rem_std,
    u32_div_rem_24_20_std;
    u32_div_rem_binary_long,
    u32_div_rem_24_20_binary_long;
);
bencher!(
    FnKind::DivRem,
    u32,
    24,
    8,
    u32_div_rem_std,
    u32_div_rem_24_8_std;
    u32_div_rem_binary_long,
    u32_div_rem_24_8_binary_long;
);
bencher!(
    FnKind::DivRem,
    u32,
    32,
    16,
    u32_div_rem_std,
    u32_div_rem_32_16_std;
    u32_div_rem_binary_long,
    u32_div_rem_32_16_binary_long;
);
// Div only
bencher!(
    FnKind::Div,
    u32,
    32,
    16,
    u32_div_rem_std,
    u32_div_32_16_std;
    u32_div_rem_binary_long,
    u32_div_32_16_binary_long;
);
// Rem only
bencher!(
    FnKind::Rem,
    u32,
    32,
    16,
    u32_div_rem_std,
    u32_rem_32_16_std;
    u32_div_rem_binary_long,
    u32_rem_32_16_binary_long;
);
bencher!(
    FnKind::DivRem,
    u64,
    48,
    38,
    u64_div_rem_std,
    u64_div_rem_48_38_std;
    u64_div_rem_binary_long,
    u64_div_rem_48_38_binary_long;
    u64_div_rem_delegate,
    u64_div_rem_48_38_delegate;
    u64_div_rem_asymmetric,
    u64_div_rem_48_38_asymmetric;
    u64_div_rem_trifecta,
    u64_div_rem_48_38_trifecta;
);
bencher!(
    FnKind::DivRem,
    u64,
    48,
    16,
    u64_div_rem_std,
    u64_div_rem_48_16_std;
    u64_div_rem_binary_long,
    u64_div_rem_48_16_binary_long;
    u64_div_rem_delegate,
    u64_div_rem_48_16_delegate;
    u64_div_rem_asymmetric,
    u64_div_rem_48_16_asymmetric;
    u64_div_rem_trifecta,
    u64_div_rem_48_16_trifecta;
);
bencher!(
    FnKind::DivRem,
    u64,
    64,
    32,
    u64_div_rem_std,
    u64_div_rem_64_32_std;
    u64_div_rem_binary_long,
    u64_div_rem_64_32_binary_long;
    u64_div_rem_delegate,
    u64_div_rem_64_32_delegate;
    u64_div_rem_asymmetric,
    u64_div_rem_64_32_asymmetric;
    u64_div_rem_trifecta,
    u64_div_rem_64_32_trifecta;
);
bencher!(
    FnKind::DivRem,
    u128,
    96,
    70,
    u128_div_rem_std,
    u128_div_rem_96_70_std;
    u128_div_rem_binary_long,
    u128_div_rem_96_70_binary_long;
    u128_div_rem_delegate,
    u128_div_rem_96_70_delegate;
    u128_div_rem_asymmetric,
    u128_div_rem_96_70_asymmetric;
    u128_div_rem_trifecta,
    u128_div_rem_96_70_trifecta;
);
bencher!(
    FnKind::DivRem,
    u128,
    96,
    32,
    u128_div_rem_std,
    u128_div_rem_96_32_std;
    u128_div_rem_binary_long,
    u128_div_rem_96_32_binary_long;
    u128_div_rem_delegate,
    u128_div_rem_96_32_delegate;
    u128_div_rem_asymmetric,
    u128_div_rem_96_32_asymmetric;
    u128_div_rem_trifecta,
    u128_div_rem_96_32_trifecta;
);
// signed division
bencher!(
    FnKind::DivRem,
    i128,
    96,
    32,
    i128_div_rem_std,
    i128_div_rem_96_32_std;
    i128_div_rem_binary_long,
    i128_div_rem_96_32_binary_long;
    i128_div_rem_delegate,
    i128_div_rem_96_32_delegate;
    i128_div_rem_asymmetric,
    i128_div_rem_96_32_asymmetric;
    i128_div_rem_trifecta,
    i128_div_rem_96_32_trifecta;
);
bencher!(
    FnKind::DivRem,
    u128,
    128,
    64,
    u128_div_rem_std,
    u128_div_rem_128_64_std;
    u128_div_rem_delegate,
    u128_div_rem_128_64_delegate;
    u128_div_rem_asymmetric,
    u128_div_rem_128_64_asymmetric;
    u128_div_rem_trifecta,
    u128_div_rem_128_64_trifecta;
);
// 128 by 96
bencher!(
    FnKind::DivRem,
    u128,
    128,
    96,
    u128_div_rem_std,
    u128_div_rem_128_96_std;
    u128_div_rem_binary_long,
    u128_div_rem_128_96_binary_long;
    u128_div_rem_delegate,
    u128_div_rem_128_96_delegate;
    u128_div_rem_asymmetric,
    u128_div_rem_128_96_asymmetric;
    u128_div_rem_trifecta,
    u128_div_rem_128_96_trifecta;
);
// divisions with `duo` and `div` being very similar
bencher!(
    FnKind::DivRem,
    u128,
    120,
    120,
    u128_div_rem_std,
    u128_div_rem_120_120_std;
    u128_div_rem_binary_long,
    u128_div_rem_120_120_binary_long;
    u128_div_rem_delegate,
    u128_div_rem_120_120_delegate;
    u128_div_rem_asymmetric,
    u128_div_rem_120_120_asymmetric;
    u128_div_rem_trifecta,
    u128_div_rem_120_120_trifecta;
);
// bench short division by a very small div
bencher!(
    FnKind::DivRem,
    u128,
    128,
    8,
    u128_div_rem_std,
    u128_div_rem_128_8_std;
    u128_div_rem_binary_long,
    u128_div_rem_128_8_binary_long;
    u128_div_rem_delegate,
    u128_div_rem_128_8_delegate;
    u128_div_rem_asymmetric,
    u128_div_rem_128_8_asymmetric;
    u128_div_rem_trifecta,
    u128_div_rem_128_8_trifecta;
);
