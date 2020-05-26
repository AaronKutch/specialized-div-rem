macro_rules! impl_trifecta {
    (
        $unsigned_name:ident, // name of the unsigned division function
        $signed_name:ident, // name of the signed division function
        $half_division:ident, // function for division of a $uX by a $uX
        $n_h:expr, // the number of bits in $iH or $uH
        $uH:ident, // unsigned integer with half the bit width of $uX
        $uX:ident, // unsigned integer with half the bit width of $uD
        $uD:ident, // unsigned integer type for the inputs and outputs of `$unsigned_name`
        $iD:ident, // signed integer type for the inputs and outputs of `$signed_name`
        $($unsigned_attr:meta),*; // attributes for the unsigned function
        $($signed_attr:meta),* // attributes for the signed function
    ) => {
        /// Computes the quotient and remainder of `duo` divided by `div` and returns them as a
        /// tuple.
        ///
        /// This is optimized for division of two integers with bit widths twice as large
        /// as the largest hardware integer division supported. Note that some architectures supply
        /// a division of an integer larger than register size by a regular sized integer (e.x.
        /// x86_64 has an assembly instruction which can divide a 128 bit integer by a 64 bit
        /// integer). In that case, the `_asymmetric` algorithm should be used instead of this one.
        ///
        /// Note that sometimes, CPUs can have hardware division, but it is emulated in a way that
        /// is not significantly faster than custom binary long division, and/or the hardware
        /// multiplier is so slow that the alternative `_delegate` algorithm is faster. `_trifecta`
        /// depends on both the divider and multplier being fast.
        ///
        /// This is called the trifecta algorithm because it uses three main algorithms: short
        /// division for small divisors, the "mul or mul - 1" algorithm for when the divisor is
        /// large enough for the quotient to be determined to be one of two values via only one
        /// small division, and an undersubtracting long division algorithm for middle cases.
        ///
        /// # Panics
        ///
        /// When attempting to divide by zero, this function will panic.
        $(
            #[$unsigned_attr]
        )*
        pub fn $unsigned_name(duo: $uD, div: $uD) -> ($uD, $uD) {
            // wrapping_{} was used everywhere for better performance when compiling with
            // `debug-assertions = true`

            // This replicates `carrying_mul` (rust-lang rfc #2417). LLVM correctly optimizes this
            // to use a widening multiply to 128 bits.
            #[inline]
            fn carrying_mul(lhs: $uX, rhs: $uX) -> ($uX, $uX) {
                let tmp = (lhs as $uD).wrapping_mul(rhs as $uD);
                (tmp as $uX, (tmp >> ($n_h * 2)) as $uX)
            }
            #[inline]
            fn carrying_mul_add(lhs: $uX, mul: $uX, add: $uX) -> ($uX, $uX) {
                let tmp = (lhs as $uD).wrapping_mul(mul as $uD).wrapping_add(add as $uD);
                (tmp as $uX, (tmp >> ($n_h * 2)) as $uX)
            }

            // the number of bits in a $uX
            let n = $n_h * 2;

            if div == 0 {
                panic!("division by zero")
            }

            // Note that throughout this function, `lo` and `hi` refer to the high and low `n` bits
            // of a `$uD`, `0` to `3` refer to the 4 `n_h` bit parts of a `$uD`,
            // and `mid` refer to the middle two `n_h` parts.

            let div_lz = div.leading_zeros();
            let mut duo_lz = duo.leading_zeros();

            // the possible ranges of `duo` and `div` at this point:
            // `0 <= duo < 2^n_d`
            // `1 <= div < 2^n_d`

            // quotient is 0 or 1 branch
            if div_lz <= duo_lz {
                // The quotient cannot be more than 1. The highest set bit of `duo` needs to be at
                // least one place higher than `div` for the quotient to be more than 1.
                if duo >= div {
                    return (1,duo.wrapping_sub(div))
                } else {
                    return (0,duo)
                }
            }

            // `_sb` is the number of significant bits (from the ones place to the highest set bit)
            // `{2, 2^div_sb} <= duo < 2^n_d`
            // `1 <= div < {2^duo_sb, 2^(n_d - 1)}`
            // smaller division branch
            if duo_lz >= n {
                // `duo < 2^n` so it will fit in a $uX. `div` will also fit in a $uX (because of the
                // `div_lz <= duo_lz` branch) so no numerical error.
                let tmp = $half_division(duo as $uX, div as $uX);
                return (
                    tmp.0 as $uD,
                    tmp.1 as $uD
                )
            }

            // relative leading significant bits, cannot overflow because of above branches
            let rel_leading_sb = div_lz.wrapping_sub(duo_lz);

            // `{2^n, 2^div_sb} <= duo < 2^n_d`
            // `1 <= div < {2^duo_sb, 2^(n_d - 1)}`
            // short division branch
            if div_lz >= (n + $n_h) {
                // `1 <= div < {2^duo_sb, 2^n_h}`
                let duo_hi = (duo >> n) as $uX;
                let div_0 = div as $uH as $uX;
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
            }

            // `{2^n, 2^div_sb} <= duo < 2^n_d`
            // `2^n_h <= div < {2^duo_sb, 2^(n_d - 1)}`
            // `mul` or `mul - 1` branch
            if rel_leading_sb < $n_h {
                // proof that `quo` can only be `mul` or `mul - 1`.
                // disclaimer: this is not rigorous
                //
                // We are trying to find the quotient, `quo`.
                // 1. quo = duo / div. (definition)
                //
                // `shift` is the number of bits not in the higher `n` significant bits of `duo`
                // 2. shift = n - duo_lz. (definition)
                // 3. duo_sig_n == duo / 2^shift. (definition)
                // 4. div_sig_n == div / 2^shift. (definition)
                // Note: the `_sig_n` (which in my notation usually means that the variable uses the
                // most significant `n` bits) on `div_sig_n` is a misnomer, because instead of using
                // a shift by `n - div_lz`, what I do is use the `n - duo_lz` shift so that the bits
                // of (4) correspond to (3).
                //
                // this is because of the bits less significant than the sig_n bits that are cut off
                // during the bit shift
                // 5. duo_sig_n * 2^shift <= duo < (duo_sig_n + 1) * 2^shift.
                // 6. div_sig_n * 2^shift <= div < (div_sig_n + 1) * 2^shift.
                //
                // dividing each bound of (5) by each bound of (6)
                // (duo_sig_n * 2^shift) / (div_sig_n * 2^shift)
                // (duo_sig_n * 2^shift) / ((div_sig_n + 1) * 2^shift)
                // ((duo_sig_n + 1) * 2^shift) / (div_sig_n * 2^shift)
                // ((duo_sig_n + 1) * 2^shift) / ((div_sig_n + 1) * 2^shift)
                // simplifying each of these four
                // duo_sig_n / div_sig_n
                // duo_sig_n / (div_sig_n + 1)
                // (duo_sig_n + 1) / div_sig_n
                // (duo_sig_n + 1) / (div_sig_n + 1)
                // taking the smallest and the largest of these as the low and high bounds
                // and replacing `duo / div` with `quo`
                // 7. duo_sig_n / (div_sig_n + 1) <= quo < (duo_sig_n + 1) / div_sig_n
                //
                // Here is where the range restraints from the previous branches becomes necessary.
                // To help visualize what needs to happen here, is that the top `n` significant
                // bits of `duo` are being divided with the corresponding bits of `div`.
                //
                // The `rel_leading_sb < n_h` conditional on this branch makes sure that `div_sig_n`
                // is at least `2^n_h`, and the previous branches make sure that the highest bit of
                // `div_sig_n` is not the `2^(n - 1)` bit (which `duo_sig_n` has been shifted up to
                // attain).
                // 8. `2^(n - 1) <= duo_sig_n < 2^n`
                // 9. `2^n_h <= div_sig_n < 2^(n - 1)`
                // 10. mul == duo_sig_n / div_sig_n. (definition)
                //
                // Using the bounds (8) and (9) with the `duo_sig_n / (div_sig_n + 1)` term. Note
                // that on the `<` bounds we subtract 1 to make it inclusive.
                // (2^n - 1) / (2^n_h + 1) = 2^n_h - 1
                // 2^(n - 1) / 2^(n - 1) = 1
                // The other equations using the both upper bounds or both lower bounds end up with
                // values in the middle of this range.
                // 11. 1 <= mul <= 2^n_h - 1
                //
                // However, all this tells us is that `mul` can fit in a $uH.
                // We want to show that `mul - 1 <= quo < mul + 1`. What this and (7) means is that
                // the bits cut off by the shift at the beginning can only affect `quo` enough to
                // change it between two values. First, I will show that
                // `(duo_sig_n + 1) / div_sig_n` is equal to or less than `mul + 1`. We cannot
                // simply use algebra here and say that `1 / div_sig_n` is 0, because we are working
                // with truncated integer division. If the remainder is `div_sig_n - 1`, then the
                // `+ 1` can be enough to cross the boundary and increment the quotient
                // (e.x. 127/32 = 3, 128/32 = 4). Instead, we notice that in order for the remainder
                // of `(duo_sig_n + x) / div_sig_n` to be wrapped around twice, `x` must be 1 for
                // one of the wrap arounds and another `div_sig_n` added on to it to wrap around
                // again. This cannot happen even if `div_sig_n` were 1, so
                // 12. duo_sig_n / (div_sig_n + 1) <= quo < mul + 1
                //
                // Next, we want to show that `duo_sig_n / (div_sig_n + 1)` is more than or equal to
                // `mul - 1`. Putting all the combinations of bounds from (8) and (9) into (10) and
                // `duo_sig_n / (div_sig_n + 1)`,
                // (2^n - 1) / 2^n_h = 2^n_h - 1
                // (2^n - 1) / (2^n_h + 1) = 2^n_h - 1
                //
                // (2^n - 1) / (2^(n - 1) - 1) = 2
                // (2^n - 1) / (2^(n - 1) - 1 + 1) = 1
                //
                // 2^(n - 1) / 2^n_h = 2^(n_h - 1)
                // 2^(n - 1) / (2^n_h + 1) = 2^(n_h - 1) - 1
                //
                // 2^(n - 1) / (2^(n - 1) - 1) = 1
                // 2^(n - 1) / 2^(n - 1) = 1
                //
                // 13. mul - 1 <= quo < mul + 1
                //
                // In a lot of division algorithms using smaller divisions to construct a larger
                // division, we often encounter a situation where the approximate `mul` value
                // calculated from a smaller division is a few increments away from the true `quo`
                // value. In those algorithms, they have to do more divisions. Because of the fact
                // that our `quo` can only be one of two values, we can try the higher value `mul`
                // and see if `duo - (mul*div)` overflows. If it did overflow, then `quo` must be
                // `mul - 1`, and because we already calculated `duo - (mul*div)` with wrapping
                // operations, we can do simple operations without dividing again to find
                // `duo - ((mul - 1)*div) = duo - (mul*div - div) = duo + div - mul*div`. If it
                // did not overflow, then `mul` must be the answer, because the remainder of
                // `duo - ((mul - 1)*div)` is larger and farther away from overflow than
                // `duo - (mul*div)`, which we already found is not overflowing.
                //
                // Thus, we find the quotient using only an `n` sized divide to find `mul`
                // and a `n` by `d_n` sized multiply and comparison to find if `quo * mul > duo`,
                // following with adds and subtracts to get the correct values.
                let shift = n.wrapping_sub(duo_lz);
                let duo_sig_n = (duo >> shift) as $uX;
                let div_sig_n = (div >> shift) as $uX;
                let mul = $half_division(duo_sig_n, div_sig_n).0;
                // inline `n` bit by `n_d` bit multiplication and overflow check (we cannot do
                // `mul * div > duo` directly because of possibility of overflow beyond 128 bits)
                // duo cannot be more than `2^n_d - 1`, and overflow means a value more than that
                let div_lo = div as $uX;
                let div_hi = (div >> n) as $uX;
                let (tmp_lo, carry) = carrying_mul(mul,div_lo);
                let (tmp_hi, overflow) = carrying_mul_add(mul,div_hi,carry);
                let tmp = (tmp_lo as $uD) | ((tmp_hi as $uD) << n);
                // Note that the overflow cannot be more than 1, otherwise the `div` subtraction
                // would not be able to bring the remainder value below 2^n_d - 1, which
                // contradicts many things. The `& 1` is here to encourage overflow flag use.
                if ((overflow & 1) != 0) || (duo < tmp) {
                    return (
                        mul.wrapping_sub(1) as $uD,
                        duo.wrapping_add(div.wrapping_sub(tmp))
                    )
                } else {
                    return (
                        mul as $uD,
                        duo.wrapping_sub(tmp)
                    )
                }
            }

            // Undersubtracting long division algorithm.
            // Instead of clearing a minimum of 1 bit from `duo` per iteration via
            // binary long division, `n_h - 1` bits are cleared per iteration with this algorithm.
            // It is a more complicated version of regular long division. Most integer division
            // algorithms tend to guess a part of the quotient, and may have a larger quotient than
            // the true quotient (which when multiplied by `div` will "oversubtract" the original
            // dividend). They then check if the quotient was in fact too large and then have to
            // correct it. This long division algorithm has been carefully constructed to always
            // underguess the quotient by slim margins. This allows different subalgorithms
            // to be blindly jumped to without needing an extra correction step.
            //
            // The only problem is that this subalgorithm will not work
            // for many ranges of `duo` and `div`. Fortunately, the short division,
            // mul or mul - 1 algorithm, and simple divisions happen to exactly fill these gaps.
            //
            // For an example, consider the division of 76543210 by 213 and assume that `n_h`
            // is equal to two decimal digits (note: we are working with base 10 here for
            // readability).
            // The first `h_n` part of the divisor (21) is taken and is incremented by
            // 1 to prevent oversubtraction.
            // In the first step, the first `n` part of duo (7654) is divided by the 22 to make 347.
            // We remember that there was 1 extra place not in the `n_h` part of the divisor and
            // shift the 347 right by 1, in contrast to a normal long division. The 347 is
            // multiplied by the whole divisor to make 73911, and subtracted from duo to finish the
            // step.
            //    347
            //  ________
            // |76543210
            // -73911
            //   2632210
            // Two more steps are taken after this and then duo fits into `n` bits, and then a final
            // normal long division step is made.
            //        14
            //       443
            //     119
            //    347
            //  ________
            // |76543210
            // -73911
            //   2632210
            //  -25347
            //     97510
            //    -94359
            //      3151
            // The tower at the top is added together to produce the quotient, 359357 (but in the
            // actual algorithm, the quotient is progressively added to each step instead of at
            // the end).
            // In the actual algorithm below, instead of the final normal long division step, one of
            // the three other algorithms ("quotient is 0 or 1", "mul or mul - 1", or
            // "n sized division") is used.

            let mut duo = duo;

            // the number of lesser significant bits not a part of `div_sig_n_h`. Must be positive.
            let div_lesser_places = (n + $n_h).wrapping_sub(div_lz);

            // the most significant `n_h` bits of div
            let div_sig_n_h = (div >> div_lesser_places) as $uH;

            // has to be a `$uX` in case of overflow
            let div_sig_n_h_add1 = (div_sig_n_h as $uX).wrapping_add(1);

            let mut quo: $uD = 0;

            // `{2^n, 2^(div_sb + n_h)} <= duo < 2^n_d`
            // `2^n_h <= div < {2^(duo_sb - n_h), 2^n}`
            loop {
                let duo_lesser_places = n.wrapping_sub(duo_lz);
                let duo_sig_n = (duo >> duo_lesser_places) as $uX;

                if div_lesser_places <= duo_lesser_places {
                    let mul = $half_division(duo_sig_n, div_sig_n_h_add1).0 as $uD;
                    let place = duo_lesser_places.wrapping_sub(div_lesser_places);

                    //addition to the quotient
                    quo = quo.wrapping_add(mul << place);

                    //subtraction from `duo`
                    //at least `n_h - 1` bits are cleared from `duo` here
                    duo = duo.wrapping_sub(div.wrapping_mul(mul) << place);
                } else {
                    //`mul` or `mul - 1` algorithm
                    let shift = n.wrapping_sub(duo_lz);
                    let duo_sig_n = (duo >> shift) as $uX;
                    let div_sig_n = (div >> shift) as $uX;
                    let mul = $half_division(duo_sig_n, div_sig_n).0;
                    let div_lo = div as $uX;
                    let div_hi = (div >> n) as $uX;

                    let (tmp_lo, carry) = carrying_mul(mul,div_lo);
                    // The undersubtracting long division algorithm has already run once, so
                    // overflow beyond 128 bits is not possible here
                    let (tmp_hi, _) = carrying_mul_add(mul,div_hi,carry);
                    let tmp = (tmp_lo as $uD) | ((tmp_hi as $uD) << n);

                    if duo < tmp {
                        return (
                            // note that this time, the "mul or mul - 1" result is added to `quo`
                            // to get the final correct quotient
                            quo.wrapping_add(mul.wrapping_sub(1) as $uD),
                            duo.wrapping_add(
                                div.wrapping_sub(tmp)
                            )
                        )
                    } else {
                        return (
                            quo.wrapping_add(mul as $uD),
                            duo.wrapping_sub(tmp)
                        )
                    }
                }

                duo_lz = duo.leading_zeros();
                if div_lz <= duo_lz {
                    //quotient can have 0 or 1 added to it
                    if div <= duo {
                        return (
                            quo.wrapping_add(1),
                            duo.wrapping_sub(div)
                        )
                    } else {
                        return (
                            quo,
                            duo
                        )
                    }
                }

                // This can only happen if `div_sd < n` (because of previous "quo = 0 or 1"
                // branches), but it is not worth it to unroll further.
                if duo_lz >= n {
                    // simple division and addition
                    let tmp = $half_division(duo as $uX, div as $uX);
                    return (
                        quo.wrapping_add(tmp.0 as $uD),
                        tmp.1 as $uD
                    )
                }
            }
        }

        /// Computes the quotient and remainder of `duo` divided by `div` and returns them as a
        /// tuple.
        ///
        /// This is optimized for division of two integers with bit widths twice as large
        /// as the largest hardware integer division supported. Note that some architectures supply
        /// a division of an integer larger than register size by a regular sized integer (e.x.
        /// x86_64 has an assembly instruction which can divide a 128 bit integer by a 64 bit
        /// integer). In that case, the `_asymmetric` algorithm should be used instead of this one.
        ///
        /// Note that sometimes, CPUs can have hardware division, but it is emulated in a way that
        /// is not significantly faster than custom binary long division, and/or the hardware
        /// multiplier is so slow that the alternative `_delegate` algorithm is faster. `_trifecta`
        /// depends on both the divider and multplier being fast.
        ///
        /// This is called the trifecta algorithm because it uses three main algorithms: short
        /// division for small divisors, the "mul or mul - 1" algorithm for when the divisor is
        /// large enough for the quotient to be determined to be one of two values via only one
        /// small division, and an undersubtracting long division algorithm for middle cases.
        ///
        /// # Panics
        ///
        /// When attempting to divide by zero, this function will panic.
        $(
            #[$signed_attr]
        )*
        pub fn $signed_name(duo: $iD, div: $iD) -> ($iD, $iD) {
            match (duo < 0, div < 0) {
                (false, false) => {
                    let t = $unsigned_name(duo as $uD, div as $uD);
                    (t.0 as $iD, t.1 as $iD)
                },
                (true, false) => {
                    let t = $unsigned_name(duo.wrapping_neg() as $uD, div as $uD);
                    ((t.0 as $iD).wrapping_neg(), (t.1 as $iD).wrapping_neg())
                },
                (false, true) => {
                    let t = $unsigned_name(duo as $uD, div.wrapping_neg() as $uD);
                    ((t.0 as $iD).wrapping_neg(), t.1 as $iD)
                },
                (true, true) => {
                    let t = $unsigned_name(duo.wrapping_neg() as $uD, div.wrapping_neg() as $uD);
                    (t.0 as $iD, (t.1 as $iD).wrapping_neg())
                },
            }
        }
    }
}
