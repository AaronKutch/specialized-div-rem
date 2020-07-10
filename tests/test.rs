use rand::random;

#[rustfmt::skip]
use specialized_div_rem::{
    test,
    usize_leading_zeros,
    u8_div_rem_binary_long,
    i8_div_rem_binary_long,
    u16_div_rem_binary_long,
    i16_div_rem_binary_long,
    u32_div_rem_binary_long,
    i32_div_rem_binary_long,
    u32_div_rem_delegate,
    i32_div_rem_delegate,
    u64_div_rem_binary_long,
    i64_div_rem_binary_long,
    u64_div_rem_delegate,
    i64_div_rem_delegate,
    u64_div_rem_trifecta,
    i64_div_rem_trifecta,
    u64_div_rem_asymmetric,
    i64_div_rem_asymmetric,
    u128_div_rem_delegate,
    i128_div_rem_delegate,
    u128_div_rem_trifecta,
    i128_div_rem_trifecta,
    u128_div_rem_asymmetric,
    i128_div_rem_asymmetric,
};

test!(
    8,
    u8,
    i8,
    div_rem_binary_long_8,
    u8_div_rem_binary_long,
    i8_div_rem_binary_long;
);
test!(
    16,
    u16,
    i16,
    div_rem_binary_long_16,
    u16_div_rem_binary_long,
    i16_div_rem_binary_long;
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

#[test]
fn usize_leading_zeros_test() {
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
        if usize_leading_zeros(x) != (x.leading_zeros() as usize) {
            panic!(
                "x: {}, expected: {}, found: {}",
                x,
                x.leading_zeros(),
                usize_leading_zeros(x)
            );
        }
    }
}
