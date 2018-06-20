#![feature(test)]
extern crate test;
extern crate rand;
use rand::prelude::*;
use test::Bencher;
use test::black_box;
use std::{u32,u64,u128};

extern crate specialized_div_rem;
use specialized_div_rem::*;

const C: [u128; 6] = [128756765776577777777777778989897568763u128,4586787978568756634253423453454363u128,45856875663425342345345436u128,45867873423453454363u128,867834545436u128,1786u128];

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
                sum0 += u128_div_rem_long(a[i0],a[i1]).0;
                sum1 += u128_div_rem_long(a[i0],a[i1]).1;
            }
        }
        (sum0, sum1)
    });
}

//makes a function that approximates how long of the `std_and_long_bencher` benchmarks take outside of the actual division and remainder operations
macro_rules! baseline_bencher {
    ($name:ident,$ty:ty,$ty_zero:expr,$ty_one:expr) => {
        #[bench]
        fn $name(bencher: &mut Bencher) {
            let (a,b) = black_box({
                let (mut a,mut b): (Vec<$ty>,Vec<$ty>) = (Vec::new(),Vec::new());
                for _ in 0..1024 {
                    let temp0: $ty = random();
                    a.push(temp0);
                    let temp1: $ty = random();
                    if temp1 == $ty_zero {
                        b.push($ty_one);
                    } else {
                        b.push(temp1);
                    }
                }
                (a,b)
            });
            bencher.iter(|| {
                a.iter().enumerate().fold(($ty_zero,$ty_zero),|(sum0,sum1), x| {(sum0.wrapping_add(b[x.0]),sum1.wrapping_sub(b[x.0]))})
            })
        }
    };
}

baseline_bencher!(u32_baseline,u32,0u32,1u32);
baseline_bencher!(u64_baseline,u64,0u64,1u64);
baseline_bencher!(u128_baseline,u128,0u128,1u128);

enum FnKind {
    DivRem,
    Div,
    Rem
}

//produces two benchmarking functions that iterates a standard rust operation and an alternative operation on an precomputed array of 1024 random integers.The iterator is passed to a `Bencher`, and when `cargo bench` is used, it returns the time to do all 1024 operations. Note that some time is taken to lookup the array values and add the result of the operation, so subtract the appropriate `{}_baseline` values to find a closer value to how long the operations themselves actually take.
macro_rules! std_and_long_bencher {
    //$fn => the alternative operation
    //$fn_kind => this is to reduce repeated macro code, it specifies what the operation is
    //$ty => the type that is entered into the operations
    //$ty_bits => the number of bits in a `$ty`
    //$ty_zero => the zero of `$ty`
    //$ty_one => the one of `$ty`
    //$arg0_sb => the number of significant bits in argument 0 to the operation
    //$arg1_sb => the number of significant bits in argument 1 to the operation. Note: argument 1 is set to 1 if the random number generator returns zero
    //$name_std => name of the benchmarking function that uses the standard Rust operations
    //$name_long => name of the benchmarking function that uses another operation
    ($fn:ident,$fn_kind:expr,$ty:tt,$ty_bits:expr,$ty_zero:expr,$ty_one:expr,$arg0_sb:expr,$arg1_sb:expr,$name_std:ident,$name_long:ident) => {
        #[bench]
        fn $name_std(bencher: &mut Bencher) {
            let (a,b) = black_box({
                let (mut a,mut b): (Vec<$ty>,Vec<$ty>) = (Vec::new(),Vec::new());
                for _ in 0..1024 {
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
                match $fn_kind {
                    FnKind::DivRem => a.iter().enumerate().fold(($ty_zero,$ty_zero),|(sum0,sum1), x| {(sum0.wrapping_add(x.1 / b[x.0]),sum1.wrapping_add(x.1 % b[x.0]))}),
                    //sum1 just exists here to make the return types of the match arms equal
                    FnKind::Div => a.iter().enumerate().fold(($ty_zero,$ty_zero),|(sum0,sum1), x| {(sum0.wrapping_add(x.1 / b[x.0]),sum1)}),
                    FnKind::Rem => a.iter().enumerate().fold(($ty_zero,$ty_zero),|(sum0,sum1), x| {(sum0.wrapping_add(x.1 % b[x.0]),sum1)}),
                }
            })
        }

        #[bench]
        fn $name_long(bencher: &mut Bencher) {
            let (a,b) = black_box({
                let (mut a,mut b): (Vec<$ty>,Vec<$ty>) = (Vec::new(),Vec::new());
                for _ in 0..1024 {
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
                match $fn_kind {
                    FnKind::DivRem => a.iter().enumerate().fold(($ty_zero,$ty_zero),|(sum0,sum1), x| {
                        let temp = $fn(*x.1,b[x.0]);
                        (sum0.wrapping_add(temp.0),sum1.wrapping_add(temp.1))
                    }),
                    //to prevent a lot of code duplication, I used the inline div_rem functions and used only one field from them (which is exactly what is used to define the div and rem only functions).
                    FnKind::Div => a.iter().enumerate().fold(($ty_zero,$ty_zero),|(sum0,sum1), x| {(
                        sum0.wrapping_add($fn(*x.1,b[x.0]).0),sum1)}),
                    FnKind::Rem => a.iter().enumerate().fold(($ty_zero,$ty_zero),|(sum0,sum1), x| {(sum0.wrapping_add($fn(*x.1,b[x.0]).1),sum1)}),
                }
            })
        }
    };
}

std_and_long_bencher!(u32_div_rem_long,FnKind::DivRem,u32,32,0u32,1u32,32,32 - 8,u32_div_rem_all_mid_std,u32_div_rem_all_mid_long);
std_and_long_bencher!(u32_div_rem_long_inline_always,FnKind::Div,u32,32,0u32,1u32,32,32 - 8,u32_div_all_mid_std,u32_div_all_mid_long);
std_and_long_bencher!(u32_div_rem_long_inline_always,FnKind::Rem,u32,32,0u32,1u32,32,32 - 8,u32_rem_all_mid_std,u32_rem_all_mid_long);

std_and_long_bencher!(u64_div_rem_long,FnKind::DivRem,u64,64,0u64,1u64,64,64 - 16,u64_div_rem_all_mid_std,u64_div_rem_all_mid_long);
std_and_long_bencher!(u64_div_rem_long_inline_always,FnKind::Div,u64,64,0u64,1u64,64,64 - 16,u64_div_all_mid_std,u64_div_all_mid_long);
std_and_long_bencher!(u64_div_rem_long_inline_always,FnKind::Rem,u64,64,0u64,1u64,64,64 - 16,u64_rem_all_mid_std,u64_rem_all_mid_long);

std_and_long_bencher!(u128_div_rem_long,FnKind::DivRem,u128,128,0u128,1u128,128,128 - 32,u128_div_rem_all_mid_std,u128_div_rem_all_mid_long);
std_and_long_bencher!(u128_div_rem_long,FnKind::DivRem,u128,128,0u128,1u128,128,128,u128_div_rem_all_all_std,u128_div_rem_all_all_long);
std_and_long_bencher!(u128_div_rem_long,FnKind::DivRem,u128,128,0u128,1u128,128,32,u128_div_rem_all_0_std,u128_div_rem_all_0_long);
std_and_long_bencher!(u128_div_rem_long,FnKind::DivRem,u128,128,0u128,1u128,64,32,u128_div_rem_lo_0_std,u128_div_rem_lo_0_long);
std_and_long_bencher!(u128_div_rem_long,FnKind::DivRem,u128,128,0u128,1u128,32,32,u128_div_rem_0_0_std,u128_div_rem_0_0_long);
std_and_long_bencher!(u128_div_rem_long,FnKind::DivRem,u128,128,0u128,1u128,32,128,u128_div_rem_0_all_std,u128_div_rem_0_all_long);
std_and_long_bencher!(u128_div_rem_long_inline_always,FnKind::DivRem,u128,128,0u128,1u128,128,128 - 32,u128_div_rem_inline_always_all_mid_std,u128_div_rem_inline_always_all_mid_long);
std_and_long_bencher!(u128_div_rem_long_inline_always,FnKind::DivRem,u128,128,0u128,1u128,128,64,u128_div_rem_inline_always_all_lo_std,u128_div_rem_inline_always_all_lo_long);

std_and_long_bencher!(u128_div_rem_long_inline_always,FnKind::Div,u128,128,0u128,1u128,128,64,u128_div_inline_always_all_lo_std,u128_div_inline_always_all_lo_long);
std_and_long_bencher!(u128_div_rem_long_inline_always,FnKind::Rem,u128,128,0u128,1u128,128,64,u128_rem_inline_always_all_lo_std,u128_rem_inline_always_all_lo_long);