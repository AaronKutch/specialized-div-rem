macro_rules! impl_binary_shift {
    (
        $unsigned_name:ident, // name of the unsigned function
        $signed_name:ident, // name of the signed function
        $test_name:ident, // name of the test function
        $n_h:expr, // the number of bits in $iH or $uH
        $uH:ident, // unsigned integer with half the bit width of $uX
        $uX:ident, // unsigned integer with half the bit width of $uD
        $uD:ident, // unsigned integer with double the bit width of $uX
        $iD:ident, // signed version of $uD
        $($unsigned_attr:meta),*; // attributes for the unsigned function
        $($signed_attr:meta),* // attributes for the signed function
    ) => {

        /// Computes the quotient and remainder of `duo` divided by `div` and returns them as a
        /// tuple. This uses binary shift long division (unless a fast path uses a smaller division
        /// that uses some other algorithm).
        ///
        /// # Panics
        ///
        /// When attempting to divide by zero, this function will panic.
        $(
            #[$unsigned_attr]
        )*
        pub fn $unsigned_name(duo: $uD, div: $uD) -> ($uD,$uD) {
            // Note: `wrapping_` functions are used for add and subtract for better debug
            // performance, but others such as shift left do not use this because they always
            // have bounds checks anyway.

            // the number of bits in a $uX
            let n = $n_h * 2;

            let duo_lo = duo as $uX;
            let duo_hi = (duo >> n) as $uX;
            let div_lo = div as $uX;
            let div_hi = (div >> n) as $uX;

            match (div_lo == 0, div_hi == 0, duo_hi == 0) {
                (true, true, _) => {
                    panic!("division by zero")
                }
                (_,false,true) => {
                    // `duo` < `div`
                    return (0, duo)
                }
                (false, true, true) => {
                    // delegate to smaller division
                    return ((duo_lo / div_lo) as $uD, (duo_lo % div_lo) as $uD)
                }
                (false,true,false) => {
                    if (div_lo >> $n_h) == 0 {
                        // Short division of $uD by a $uH, using $uX by $uX division
                        let quo_hi = duo_hi.wrapping_div(div_lo);
                        // all of the `rem_{}`s cannot be larger than `$uH::MAX`,
                        // since `div` cannot be larger than `$uH::MAX`.
                        let rem_2 = duo_hi.wrapping_rem(div_lo) as $uH;

                        let duo_mid =
                            (((duo >> $n_h) as $uH) as $uX)
                            | ((rem_2 as $uX) << $n_h);
                        let quo_1 = duo_mid.wrapping_div(div_lo) as $uH;
                        let rem_1 = duo_mid.wrapping_rem(div_lo) as $uH;

                        let duo_lo =
                            ((duo as $uH) as $uX)
                            | ((rem_1 as $uX) << $n_h);
                        let quo_0 = duo_lo.wrapping_div(div_lo) as $uH;

                        return (
                            (quo_0 as $uD) | ((quo_1 as $uD) << $n_h) | ((quo_hi as $uD) << n),
                            (duo_lo.wrapping_rem(div_lo) as $uH) as $uD
                        )
                    } else {
                        // Binary long division of $uD by $uX. This is the same as below, but the
                        // compiler should be able to do the shifts faster. See the other long
                        // division below for more.
                        let div_lz = div_lo.leading_zeros() + n;
                        let duo_lz = duo_hi.leading_zeros();

                        // initial check for `div_lz < duo_lz` not needed here

                        let mut shift = div_lz - duo_lz;
                        let mut duo = duo;
                        // There are edge cases where `quo` needs to be n + 1 bits before the $uX by
                        // $uX division can be used, e.x. 0xFFFF_FFF8 / 0x7FFF. The first iteration
                        // is done before the loop, and the 1 bit it adds to `quo` is added at the
                        // end.
                        let mut sub = (div_lo as $uD) << shift;
                        let initial_shift = shift;

                        let quo_edge = if sub <= duo {
                            duo -= sub;
                            true
                        } else {
                            false
                        };
                        let mut quo_hi: $uX = 0;
                        loop {
                            let duo_hi = (duo >> n) as $uX;
                            if duo_hi == 0 || shift == 0 {
                                // delegate what remains to $uX by $uX division
                                let duo_lo = duo as $uX;
                                let (quo_lo, rem_lo) = (duo_lo / div_lo, duo_lo % div_lo);
                                return (
                                    (quo_lo as $uD)
                                    | ((quo_hi as $uD) << shift)
                                    | ((quo_edge as $uD) << initial_shift),
                                    rem_lo as $uD
                                )
                            }
                            shift -= 1;
                            quo_hi <<= 1;
                            sub >>= 1;
                            if sub <= duo {
                                duo -= sub;
                                quo_hi |= 1;
                            }
                        }
                    }
                }
                (_,false,false) => {
                    // Full $uD binary long division. Use `leading_zeros` on the first round,
                    // because we assume that the average usage of division has arguments that
                    // are random but have a significant number of leading zero bits. Doing
                    // `leading_zeros` for every round would be very expensive, especially for
                    // CPUs without a native count leading zeros instruction, but doing it just
                    // for the first round is advantageous for both performance of the common
                    // case and for code simplicity. Note that many benchmarks use the full
                    // `n_d` bits for `duo`, and the benchmarks with several bits less have a
                    // good performance increase.

                    // The "mul or mul - 1" algorithm is not used here, since CPUs without division
                    // hardware also probably do not have fast multiplication hardware.

                    let div_lz = div_hi.leading_zeros();
                    let duo_lz = duo_hi.leading_zeros();

                    if div_lz < duo_lz {
                        return (0, duo)
                    }

                    // Figures out how far `div` should be shifted to align most significant
                    // bits
                    let mut shift = div_lz - duo_lz;
                    let mut duo = duo;
                    let mut quo = 0;
                    let mut sub = div << shift;
                    loop {
                        if sub <= duo {
                            // the subtraction will not overflow
                            duo -= sub;
                            quo |= 1;
                        }
                        let duo_hi = (duo >> n) as $uX;
                        if duo_hi == 0 || shift == 0 {
                            return (quo << shift, duo)
                        }
                        shift -= 1;
                        sub >>= 1;
                        quo <<= 1;
                    }
                }
            }
        }

        /// Computes the quotient and remainder of `duo` divided by `div` and returns them as a
        /// tuple. This uses binary shift long division (unless a fast path uses a smaller division
        /// that uses some other algorithm).
        ///
        /// # Panics
        ///
        /// When attempting to divide by zero, this function will panic.
        $(
            #[$signed_attr]
        )*
        pub fn $signed_name(duo: $iD, div: $iD) -> ($iD,$iD) {
            match (duo < 0, div < 0) {
                (false,false) => {
                    let t = $unsigned_name(duo as $uD,div as $uD);
                    (t.0 as $iD,t.1 as $iD)
                },
                (true,false) => {
                    let t = $unsigned_name(duo.wrapping_neg() as $uD,div as $uD);
                    ((t.0 as $iD).wrapping_neg(),(t.1 as $iD).wrapping_neg())
                },
                (false,true) => {
                    let t = $unsigned_name(duo as $uD,div.wrapping_neg() as $uD);
                    ((t.0 as $iD).wrapping_neg(),t.1 as $iD)
                },
                (true,true) => {
                    let t = $unsigned_name(duo.wrapping_neg() as $uD,div.wrapping_neg() as $uD);
                    (t.0 as $iD,(t.1 as $iD).wrapping_neg())
                },
            }
        }

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
                                $unsigned_name(lhs1,rhs1),
                                (
                                    lhs1.wrapping_div(rhs1),
                                    lhs1.wrapping_rem(rhs1)
                                )
                            );
                            assert_eq!(
                                $signed_name(lhs1 as $iD,rhs1 as $iD),
                                (
                                    (lhs1 as $iD).wrapping_div(rhs1 as $iD),
                                    (lhs1 as $iD).wrapping_rem(rhs1 as $iD)
                                )
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
            // creates a mask for indexing the bits of a $uD
            let bit_selector_max = core::$uX::MAX.count_ones() - 1;
            for _ in 0..10_000_000 {
                let r0: u32 = bit_selector_max & random::<u32>();
                ones = 0;
                for _ in 0..r0 {
                    ones <<= 1;
                    ones |= 1;
                }
                let r1: u32 = bit_selector_max & random::<u32>();
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
                    );
                }
            }
        }
    }
}
