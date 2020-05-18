#![cfg_attr(feature = "no_std", no_std)]
#![cfg_attr(feature = "asm", feature(llvm_asm))]

// it would be annoying to convert the test function in the macro to one
// that could be used in a test module
#[cfg(test)]
extern crate rand;

#[macro_use]
mod binary_long;

#[macro_use]
mod delegate;

#[macro_use]
mod trifecta;

#[macro_use]
mod asymmetric;

/// This function is unsafe, because if the quotient of `duo` and `div` does not
/// fit in a `u64`, a floating point exception is thrown.
#[cfg(all(target_arch = "x86_64", feature = "asm"))]
#[inline]
unsafe fn u128_by_u64_div_rem(duo: u128, div: u64) -> (u64, u64) {
    let quo: u64;
    let rem: u64;
    let duo_lo = duo as u64;
    let duo_hi = (duo >> 64) as u64;
    llvm_asm!("divq $4"
        : "={rax}"(quo), "={rdx}"(rem)
        : "{rax}"(duo_lo), "{rdx}"(duo_hi), "r"(div)
        : "rax", "rdx"
    );
    return (quo, rem);
}

// for when the $uD by $uX assembly function cannot be called
#[cfg(any(not(target_arch = "x86_64"), not(feature = "asm")))]
#[inline]
unsafe fn u128_by_u64_div_rem(duo: u128, div: u64) -> (u64, u64) {
    ((duo / (div as u128)) as u64, (duo % (div as u128)) as u64)
}
#[inline]
unsafe fn u64_by_u32_div_rem(duo: u64, div: u32) -> (u32, u32) {
    ((duo / (div as u64)) as u32, (duo % (div as u64)) as u32)
}
#[inline]
unsafe fn u32_by_u16_div_rem(duo: u32, div: u16) -> (u16, u16) {
    ((duo / (div as u32)) as u16, (duo % (div as u32)) as u16)
}

#[inline]
fn u16_by_u16_div_rem(duo: u16, div: u16) -> (u16, u16) {
    (duo / div, duo % div)
}
#[inline]
fn u32_by_u32_div_rem(duo: u32, div: u32) -> (u32, u32) {
    (duo / div, duo % div)
}
#[inline]
fn u64_by_u64_div_rem(duo: u64, div: u64) -> (u64, u64) {
    (duo / div, duo % div)
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
                use rand::random;
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
                                    /*println!(
                                        "lhs:{} rhs:{} expected:({}, {}) found:({},{})",
                                        lhs1,
                                        rhs1,
                                        lhs1.wrapping_div(rhs1),
                                        lhs1.wrapping_rem(rhs1),
                                        $unsigned_name(lhs1,rhs1).0,
                                        $unsigned_name(lhs1,rhs1).1
                                    );*/
                                    panic!();
                                }
                                if $signed_name(lhs1 as $iX,rhs1 as $iX) !=
                                    (
                                        (lhs1 as $iX).wrapping_div(rhs1 as $iX),
                                        (lhs1 as $iX).wrapping_rem(rhs1 as $iX)
                                    ) {
                                    /*println!(
                                        "lhs:{} rhs:{} expected:({}, {}) found:({},{})",
                                        lhs1,
                                        rhs1,
                                        lhs1.wrapping_div(rhs1),
                                        lhs1.wrapping_rem(rhs1),
                                        $signed_name(lhs1 as $iX,rhs1 as $iX).0,
                                        $signed_name(lhs1 as $iX,rhs1 as $iX).1
                                    );*/
                                    panic!();
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
// `u128_div_rem_trifecta`.

// for completeness of smaller divisions for small CPUs
impl_binary_long!(
    u8_div_rem_binary_long,
    i8_div_rem_binary_long,
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
impl_trifecta!(
    u32_div_rem_trifecta,
    i32_div_rem_trifecta,
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
    div_rem_trifecta_32,
    u32_div_rem_trifecta,
    i32_div_rem_trifecta;
    div_rem_asymmetric_32,
    u32_div_rem_asymmetric,
    i32_div_rem_asymmetric
);

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

impl_binary_long!(
    u128_div_rem_binary_long,
    i128_div_rem_binary_long,
    128,
    u128,
    i128,
    inline;
    inline
);
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
    div_rem_binary_long_128,
    u128_div_rem_binary_long,
    i128_div_rem_binary_long;
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
