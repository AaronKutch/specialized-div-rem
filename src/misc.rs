//! miscellanious functions and macros used in the rest of the crate

/// Creates multiple intensive test functions for division functions of a certain size
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
                                    let (quo, rem) = $unsigned_name(lhs, rhs);
                                    if lhs != rhs.wrapping_mul(quo).wrapping_add(rem) {
                                        panic!(
                                            "unsigned division function failed with lhs:{} rhs:{} \
                                            expected:({}, {}) found:({}, {})",
                                            lhs,
                                            rhs,
                                            lhs.wrapping_div(rhs),
                                            lhs.wrapping_rem(rhs),
                                            $unsigned_name(lhs, rhs).0,
                                            $unsigned_name(lhs, rhs).1
                                        );
                                    }

                                    // test the signed division function also
                                    let lhs = lhs as $iX;
                                    let rhs = rhs as $iX;
                                    let (quo, rem) = $signed_name(lhs, rhs);
                                    if lhs != rhs.wrapping_mul(quo).wrapping_add(rem) {
                                        panic!(
                                            "signed division function failed with lhs:{} rhs:{} \
                                            expected:({}, {}) found:({}, {})",
                                            lhs,
                                            rhs,
                                            lhs.wrapping_div(rhs),
                                            lhs.wrapping_rem(rhs),
                                            $signed_name(lhs, rhs).0,
                                            $signed_name(lhs, rhs).1
                                        );
                                    }
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
                    // randomly OR, AND, and XOR randomly sized and shifted continuous strings of
                    // ones with `lhs` and `rhs`. XOR is performed most often because OR and AND
                    // tend to be destructive.
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
                    // This results in excellent fuzzing entropy such as:
                    // lhs: 00101011110101010101010101010000 rhs: 11111111100001111110111111111111
                    // lhs: 01110101000101010100000000000101 rhs: 11111111100001111110111111111111
                    // lhs: 00000000000000000001000000000000 rhs: 11111111100001111110111111111111
                    // lhs: 00000000000000000001000000000000 rhs: 11111111100011011111111111111111
                    // lhs: 00000000000000000010111111100000 rhs: 00000000000000000000101000000000
                    // lhs: 00000000000000000010111111100000 rhs: 10101000000000000000011101101010
                    // lhs: 00000000000000000010000001100000 rhs: 11111101010101000000011101111111
                    // lhs: 10000000000000101010101011101010 rhs: 11111101010101000000011101111000
                    if rhs != 0 {
                        let (quo, rem) = $unsigned_name(lhs,rhs);
                        if lhs != rhs.wrapping_mul(quo).wrapping_add(rem) {
                            panic!(
                                "unsigned division function failed with lhs:{} rhs:{} \
                                expected:({}, {}) found:({}, {})",
                                lhs,
                                rhs,
                                lhs.wrapping_div(rhs),
                                lhs.wrapping_rem(rhs),
                                $unsigned_name(lhs, rhs).0,
                                $unsigned_name(lhs, rhs).1
                            );
                        }
                        // `$signed_name` has already been tested plenty, but the sign bit is set
                        // half the time, so this doubles the work of one fuzzing round
                        let lhs = lhs as $iX;
                        let rhs = rhs as $iX;
                        let (quo, rem) = $signed_name(lhs, rhs);
                        if lhs != rhs.wrapping_mul(quo).wrapping_add(rem) {
                            panic!(
                                "signed division function failed with lhs:{} rhs:{} \
                                expected:({}, {}) found:({}, {})",
                                lhs,
                                rhs,
                                lhs.wrapping_div(rhs),
                                lhs.wrapping_rem(rhs),
                                $signed_name(lhs, rhs).0,
                                $signed_name(lhs, rhs).1
                            );
                        }
                    }
                }
            }
        )+
    }
}

macro_rules! impl_normalization_shift {
    (
        $name:ident, // name of the normalization shift function
        // boolean for if `$uX::leading_zeros` should be used (if an architecture does not have a
        // hardware instruction for `usize::leading_zeros`, then this should be `true`)
        $use_lz:ident,
        $n:tt, // the number of bits in a $iX or $uX
        $uX:ident, // unsigned integer type for the inputs of `$name`
        $iX:ident, // signed integer type for the inputs of `$name`
        $($unsigned_attr:meta),* // attributes for the function
    ) => {
        /// Finds the shift left that the divisor `div` would need to be normalized for a binary
        /// long division step with the dividend `duo`. NOTE: This function assumes that these edge
        /// cases have been handled before reaching it:
        /// `
        /// if div == 0 {
        ///     panic!("attempt to divide by zero")
        /// }
        /// if duo < div {
        ///     return (0, duo)
        /// }
        /// `
        ///
        /// Normalization is defined as (where `shl` is the output of this function):
        /// `
        /// if duo.leading_zeros() != (div << shl).leading_zeros() {
        ///     // If the most significant bits of `duo` and `div << shl` are not in the same place,
        ///     // then `div << shl` has one more leading zero than `duo`.
        ///     assert_eq!(duo.leading_zeros() + 1, (div << shl).leading_zeros());
        ///     // Also, `2*(div << shl)` is not more than `duo` (otherwise the first division step
        ///     // would not be able to clear the msb of `duo`)
        ///     assert!(duo < (div << (shl + 1)));
        /// }
        /// if full_normalization {
        ///     // Some algorithms do not need "full" normalization, which means that `duo` is
        ///     // larger than `div << shl` when the most significant bits are aligned.
        ///     assert!((div << shl) <= duo);
        /// }
        /// `
        ///
        /// Note: If the software bisection algorithm is being used in this function, it happens
        /// that full normalization always occurs, so be careful that new algorithms are not
        /// invisibly depending on this invariant when `full_normalization` is set to `false`.
        $(
            #[$unsigned_attr]
        )*
        fn $name(duo: $uX, div: $uX, full_normalization: bool) -> usize {
            // We have to find the leading zeros of `div` to know where its msb (most significant
            // set bit) is to even begin binary long division. It is also good to know where the msb
            // of `duo` is so that useful work can be started instead of shifting `div` for all
            // possible quotients (many division steps are wasted if `duo.leading_zeros()` is large
            // and `div` starts out being shifted all the way to the msb). Aligning the msbs of
            // `div` and `duo` could be done by shifting `div` left by
            // `div.leading_zeros() - duo.leading_zeros()`, but some CPUs without division hardware
            // also do not have single instructions for calculating `leading_zeros`. Instead of
            // software doing two bisections to find the two `leading_zeros`, we do one bisection to
            // find `div.leading_zeros() - duo.leading_zeros()` without actually knowing either of
            // the leading zeros values.

            let mut shl: usize;
            if $use_lz {
                shl = (div.leading_zeros() - duo.leading_zeros()) as usize;
                if full_normalization {
                    if duo < (div << shl) {
                        // when the msb of `duo` and `div` are aligned, the resulting `div` may be
                        // larger than `duo`, so we decrease the shift by 1.
                        shl -= 1;
                    }
                }
            } else {
                let mut test = duo;
                shl = 0usize;
                let mut lvl = $n >> 1;
                loop {
                    let tmp = test >> lvl;
                    // It happens that a final `duo < (div << shl)` check is not needed, because the
                    // `div <= tmp` check insures that the msb of `test` never passes the msb of
                    // `div`, and any set bits shifted off the end of `test` would still keep
                    // `div <= tmp` true.
                    if div <= tmp {
                        test = tmp;
                        shl += lvl;
                    }
                    // narrow down bisection
                    lvl >>= 1;
                    if lvl == 0 {
                        break
                    }
                }
            }
            // tests the invariants that should hold before beginning binary long division
            /*
            if full_normalization {
                assert!((div << shl) <= duo);
            }
            if duo.leading_zeros() != (div << shl).leading_zeros() {
                assert_eq!(duo.leading_zeros() + 1, (div << shl).leading_zeros());
                assert!(duo < (div << (shl + 1)));
            }
            */
            shl
        }
    }
}
