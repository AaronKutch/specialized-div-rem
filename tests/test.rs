use rand::random;

#[rustfmt::skip]
use specialized_div_rem::{
    test,
    test_div_by_zero,
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
    u128_div_asymmetric,
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

test_div_by_zero!(
    dbz0, u8_div_rem_binary_long;
    dbz1, i8_div_rem_binary_long;
    dbz2, u16_div_rem_binary_long;
    dbz3, i16_div_rem_binary_long;
    dbz4, u32_div_rem_binary_long;
    dbz5, i32_div_rem_binary_long;
    dbz6, u32_div_rem_delegate;
    dbz7, i32_div_rem_delegate;
    dbz8, u64_div_rem_binary_long;
    dbz9, i64_div_rem_binary_long;
    dbz10, u64_div_rem_delegate;
    dbz11, i64_div_rem_delegate;
    dbz12, u64_div_rem_trifecta;
    dbz13, i64_div_rem_trifecta;
    dbz14, u64_div_rem_asymmetric;
    dbz15, i64_div_rem_asymmetric;
    dbz16, u128_div_rem_delegate;
    dbz17, i128_div_rem_delegate;
    dbz18, u128_div_rem_trifecta;
    dbz19, i128_div_rem_trifecta;
    dbz20, u128_div_rem_asymmetric;
    dbz21, i128_div_rem_asymmetric;
);

#[test]
fn sanity_test() {
    assert_eq!(u128_div_asymmetric(1337 << 63, 42), 293610676506543696554);
}
