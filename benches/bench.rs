#![feature(test)]

extern crate test;
use rand::prelude::*;
use test::{black_box, Bencher};

use specialized_div_rem::*;

/// Calculates `specialized_div_rem::leading_zeros` 32 times with randomized operands with a random
/// number of leading zeros
#[bench]
fn usize_leading_zeros_random(bencher: &mut Bencher) {
    let v: Vec<usize> = black_box({
        let mut v = Vec::new();
        for _ in 0..32 {
            v.push(random::<usize>() & (usize::MAX >> (random::<u32>() % usize::MAX.count_ones())));
        }
        v
    });
    bencher.iter(|| v.iter().fold(0, |s, x| s + usize_leading_zeros(*x)))
}

// whatever Rust is using for the `/` and `%` operators
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

/// This macro can create multiple benchmarking functions that run 8 pairs of random integers
/// through a division function. Two masks are applied to `duo` and `div` for testing different
/// ranges of integers.
macro_rules! bencher {
    (
        // the type that is entered into the operations
        $ty:tt,
        // the size of the mask that is applied to a random number to make the dividend
        $arg0_sb:expr,
        // the size of the mask that is applied to a random number to make the divisor
        // Note: the divisor is set to 1 if the random number generator returns zero
        $arg1_sb:expr,
        // name of division function used and corresponding test
        $($fn_div_rem:ident, $test_name:ident);+;
    ) => {
        $(
            #[bench]
            fn $test_name(bencher: &mut Bencher) {
                let n = std::$ty::MAX.count_ones();
                let lhs = random::<$ty>() & ($ty::MAX >> (n - $arg0_sb));
                let mut rhs = random::<$ty>() & ($ty::MAX >> (n - $arg1_sb));
                if rhs == 0 {
                    rhs = 1;
                }
                bencher.iter(|| {
                    black_box($fn_div_rem(black_box(lhs), black_box(rhs)))
                })
            }
        )+
    };
}

// These simulate the most common cases
bencher!(
    u32,
    24,
    20,
    u32_div_rem_std,
    u32_div_rem_24_20_std;
    u32_div_rem_binary_long,
    u32_div_rem_24_20_binary_long;
);
bencher!(
    u32,
    24,
    8,
    u32_div_rem_std,
    u32_div_rem_24_8_std;
    u32_div_rem_binary_long,
    u32_div_rem_24_8_binary_long;
);
bencher!(
    u32,
    32,
    16,
    u32_div_rem_std,
    u32_div_rem_32_16_std;
    u32_div_rem_binary_long,
    u32_div_rem_32_16_binary_long;
);

bencher!(
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
    u128,
    96,
    70,
    u128_div_rem_std,
    u128_div_rem_96_70_std;
    u128_div_rem_delegate,
    u128_div_rem_96_70_delegate;
    u128_div_rem_asymmetric,
    u128_div_rem_96_70_asymmetric;
    u128_div_rem_trifecta,
    u128_div_rem_96_70_trifecta;
);
bencher!(
    u128,
    96,
    32,
    u128_div_rem_std,
    u128_div_rem_96_32_std;
    u128_div_rem_delegate,
    u128_div_rem_96_32_delegate;
    u128_div_rem_asymmetric,
    u128_div_rem_96_32_asymmetric;
    u128_div_rem_trifecta,
    u128_div_rem_96_32_trifecta;
);

// signed division
bencher!(
    i128,
    96,
    32,
    i128_div_rem_std,
    i128_div_rem_96_32_std;
    i128_div_rem_delegate,
    i128_div_rem_96_32_delegate;
    i128_div_rem_asymmetric,
    i128_div_rem_96_32_asymmetric;
    i128_div_rem_trifecta,
    i128_div_rem_96_32_trifecta;
);
bencher!(
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
    u128,
    128,
    96,
    u128_div_rem_std,
    u128_div_rem_128_96_std;
    u128_div_rem_delegate,
    u128_div_rem_128_96_delegate;
    u128_div_rem_asymmetric,
    u128_div_rem_128_96_asymmetric;
    u128_div_rem_trifecta,
    u128_div_rem_128_96_trifecta;
);

// divisions with `duo` and `div` being very similar
bencher!(
    u128,
    120,
    120,
    u128_div_rem_std,
    u128_div_rem_120_120_std;
    u128_div_rem_delegate,
    u128_div_rem_120_120_delegate;
    u128_div_rem_asymmetric,
    u128_div_rem_120_120_asymmetric;
    u128_div_rem_trifecta,
    u128_div_rem_120_120_trifecta;
);

// bench short division by a very small div
bencher!(
    u128,
    128,
    8,
    u128_div_rem_std,
    u128_div_rem_128_8_std;
    u128_div_rem_delegate,
    u128_div_rem_128_8_delegate;
    u128_div_rem_asymmetric,
    u128_div_rem_128_8_asymmetric;
    u128_div_rem_trifecta,
    u128_div_rem_128_8_trifecta;
);
