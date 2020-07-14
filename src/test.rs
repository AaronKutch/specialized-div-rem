/// Creates multiple intensive test functions for division functions of a certain size
#[macro_export]
macro_rules! test {
    (
        $n:expr, // the number of bits in a $iX or $uX
        $uX:ident, // unsigned integer that will be shifted
        $iX:ident, // signed version of $uX
        // list of triples of the test name, the unsigned division function, and the signed
        // division function
        $($test_name:ident, $unsigned_name:ident, $signed_name:ident);+;
    ) => {
        $(
            #[test]
            fn $test_name() {
                fn assert_invariants(lhs: $uX, rhs: $uX) {
                    let (quo, rem) = $unsigned_name(lhs, rhs);
                    if rhs <= rem || (lhs != rhs.wrapping_mul(quo).wrapping_add(rem)) {
                        panic!(
                            "unsigned division function failed with lhs:{} rhs:{} \
                            expected:({}, {}) found:({}, {})",
                            lhs,
                            rhs,
                            lhs.wrapping_div(rhs),
                            lhs.wrapping_rem(rhs),
                            quo,
                            rem
                        );
                    }

                    // test the signed division function also
                    let lhs = lhs as $iX;
                    let rhs = rhs as $iX;
                    let (quo, rem) = $signed_name(lhs, rhs);
                    // We cannot just test that
                    // `lhs == rhs.wrapping_mul(quo).wrapping_add(rem)`, but also
                    // need to make sure the remainder isn't larger than the divisor
                    // and has the correct sign.
                    let incorrect_rem = if rem == 0 {
                        false
                    } else if rhs == $iX::MIN {
                        // `rhs.wrapping_abs()` would overflow, so handle this case
                        // separately.
                        (lhs.is_negative() != rem.is_negative()) || (rem == $iX::MIN)
                    } else {
                        (lhs.is_negative() != rem.is_negative())
                        || (rhs.wrapping_abs() <= rem.wrapping_abs())
                    };
                    if incorrect_rem || lhs != rhs.wrapping_mul(quo).wrapping_add(rem) {
                        panic!(
                            "signed division function failed with lhs:{} rhs:{} \
                            expected:({}, {}) found:({}, {})",
                            lhs,
                            rhs,
                            lhs.wrapping_div(rhs),
                            lhs.wrapping_rem(rhs),
                            quo,
                            rem
                        );
                    }
                }

                // Brute force fuzzer that checks all possible single continuous strings of ones
                // (e.x. 0b00111000, 0b11110000, 0b01111110). This test is critical for finding
                // corner cases that the randomized fuzzer may miss.

                // This is reversed so that small values appear first, which helps development
                for lhs_len in (0..$n).rev() {
                    for lhs_shift in 0..=lhs_len {
                        for rhs_len in (0..$n).rev() {
                            for rhs_shift in 0..=rhs_len {
                                let lhs = (!0 >> lhs_len) << lhs_shift;
                                let rhs = (!0 >> rhs_len) << rhs_shift;
                                if rhs != 0 {
                                    assert_invariants(lhs, rhs);
                                }
                            }
                        }
                    }
                }

                // Specially designed random fuzzer
                let mut lhs: $uX = 0;
                let mut rhs: $uX = 0;
                // all ones constant
                let ones: $uX = !0;
                // Alternating ones and zeros (e.x. 0b1010101010101010). This catches second-order
                // problems that might occur for algorithms with two modes of operation (potentially
                // there is some invariant that can be broken for large `duo` and maintained via
                // alternating between modes, breaking the algorithm when it reaches the end).
                let mut alt_ones: $uX = 1;
                for _ in 0..($n / 2) {
                    alt_ones <<= 2;
                    alt_ones |= 1;
                }
                // creates a mask for indexing the bits of the type
                let bit_indexing_mask = $n - 1;
                for _ in 0..1_000_000 {
                    // Randomly OR, AND, and XOR randomly sized and shifted continuous strings of
                    // ones with `lhs` and `rhs`. XOR is performed most often because OR and AND
                    // tend to be destructive. This results in excellent fuzzing entropy such as:
                    // lhs: 00101011110101010101010101010000 rhs: 11111111100001111110111111111111
                    // lhs: 01110101000101010100000000000101 rhs: 11111111100001111110111111111111
                    // lhs: 00000000000000000001000000000000 rhs: 11111111100001111110111111111111
                    // lhs: 00000000000000000001000000000000 rhs: 11111111100011011111111111111111
                    // lhs: 00000000000000000010111111100000 rhs: 00000000000000000000101000000000
                    // lhs: 00000000000000000010111111100000 rhs: 10101000000000000000011101101010
                    // lhs: 00000000000000000010000001100000 rhs: 11111101010101000000011101111111
                    // lhs: 10000000000000101010101011101010 rhs: 11111101010101000000011101111000
                    // The msb is set half of the time by the fuzzer, but `assert_invariants` tests
                    // both the signed and unsigned functions.
                    let r0: u32 = bit_indexing_mask & random::<u32>();
                    let r1: u32 = bit_indexing_mask & random::<u32>();
                    let mask = ones.wrapping_shr(r0).rotate_left(r1);
                    match (random(), random(), random()) {
                        (false, false, false) => lhs |= mask,
                        (false, false, true) => lhs &= mask,
                        (false, true, _) => lhs ^= mask,
                        (true, false, false) => rhs |= mask,
                        (true, false, true) => rhs &= mask,
                        (true, true, _) => rhs ^= mask,
                    }
                    // do the same for alternating ones and zeros
                    let r0: u32 = bit_indexing_mask & random::<u32>();
                    let r1: u32 = bit_indexing_mask & random::<u32>();
                    let mask = alt_ones.wrapping_shr(r0).rotate_left(r1);
                    match (random(), random(), random()) {
                        (false, false, false) => lhs |= mask,
                        (false, false, true) => lhs &= mask,
                        (false, true, _) => lhs ^= mask,
                        (true, false, false) => rhs |= mask,
                        (true, false, true) => rhs &= mask,
                        (true, true, _) => rhs ^= mask,
                    }

                    if rhs != 0 {
                        assert_invariants(lhs, rhs);
                    }
                }
            }
        )+
    }
}
