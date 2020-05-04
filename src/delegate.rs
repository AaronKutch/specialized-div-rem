macro_rules! impl_delegate {
    (
        $unsigned_name:ident, // name of the unsigned function
        $signed_name:ident, // name of the signed function
        $half_division:ident, // function for division of a $uX by a $uX
        $n_h:expr, // the number of bits in $iH or $uH
        $uH:ident, // unsigned integer with half the bit width of $uX
        $uX:ident, // unsigned integer with half the bit width of $uD
        $uD:ident, // unsigned integer with double the bit width of $uX
        $iD:ident, // signed version of $uD
        $($unsigned_attr:meta),*; // attributes for the unsigned function
        $($signed_attr:meta),* // attributes for the signed function
    ) => {

        /// Computes the quotient and remainder of `duo` divided by `div` and returns them as a
        /// tuple.
        ///
        /// This uses binary shift long division, but if it can delegates work to a smaller
        /// division. This function is used for CPUs with a register size smaller than the division
        /// size, and that do not have fast multiplication or division hardware. For CPUs with a
        /// register size equal to the division size, the `_binary_long` functions are probably
        /// faster.
        ///
        /// # Panics
        ///
        /// When attempting to divide by zero, this function will panic.
        $(
            #[$unsigned_attr]
        )*
        pub fn $unsigned_name(duo: $uD, div: $uD) -> ($uD,$uD) {
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
                (_, false, true) => {
                    // `duo` < `div`
                    return (0, duo)
                }
                (false, true, true) => {
                    // delegate to smaller division
                    let tmp = $half_division(duo_lo, div_lo);
                    return (tmp.0 as $uD, tmp.1 as $uD)
                }
                (false, true, false) => {
                    if (div_lo >> $n_h) == 0 {
                        // Short division of $uD by a $uH, using $uX by $uX division
                        let div_0 = div_lo as $uH as $uX;
                        let (quo_hi, rem_3) = $half_division(duo_hi, div_0);

                        let duo_mid =
                            ((duo >> $n_h) as $uH as $uX)
                            | (rem_3 << $n_h);
                        let (quo_1, rem_2) = $half_division(duo_mid, div_0);

                        let duo_lo =
                            (duo as $uH as $uX)
                            | (rem_2 << $n_h);
                        let (quo_0, rem_1) = $half_division(duo_lo, div_0);

                        return (
                            (quo_0 as $uD)
                            | ((quo_1 as $uD) << $n_h)
                            | ((quo_hi as $uD) << n),
                            rem_1 as $uD
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
                                let tmp = $half_division(duo_lo, div_lo);
                                let (quo_lo, rem_lo) = (tmp.0, tmp.1);
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
                (_, false, false) => {
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
                            let duo_hi = (duo >> n) as $uX;
                            if duo_hi == 0 {
                                return (quo << shift, duo)
                            }
                        }
                        
                        if shift == 0 {
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
        /// tuple.
        ///
        /// This uses binary shift long division, but if it can delegates work to a smaller
        /// division. This function is used for CPUs with a register size smaller than the division
        /// size, and that do not have fast multiplication or division hardware. For CPUs with a
        /// register size equal to the division size, the `_binary_long` functions are probably
        /// faster.
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
    }
}
