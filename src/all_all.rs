/// Generates a function that returns the quotient and remainder of unsigned integer division of
/// `duo` by `div`. The function uses 3 different algorithms (and several conditionals for simple
/// cases) that handle almost all numerical magnitudes efficiently.
macro_rules! impl_div_rem {
    (
        $unsigned_name:ident, //name of the unsigned function
        $signed_name:ident, //name of the signed function
        $test_name:ident, //name of the test function
        //Note: $n_h is used in bit shifts, so use a type that generates the best assembly for the target machine
        $n_h:expr, //the number of bits in $iH or $uH
        $uH:ident, //unsigned integer with half the bit width of $uX
        $uX:ident, //the largest division instruction that this function calls operates on this
        $uD:ident, //unsigned integer with double the bit width of $uX
        $iD:ident, //signed version of $uD
        $($unsigned_attr:meta),*; //attributes for the unsigned function
        $($signed_attr:meta),* //attributes for the signed function
    ) => {
        //wrapping_{} was used everywhere for better performance when compiling with `debug-assertions = true`
        $(
            #[$unsigned_attr]
        )*
        pub fn $unsigned_name(duo: $uD, div: $uD) -> ($uD,$uD) {
            //TODO if and when `carrying_mul` (rust-lang rfc #2417) is stabilized, this can be fixed
            #[inline(always)]
            pub fn carrying_mul(lhs: $uX, rhs: $uX) -> ($uX, $uX) {
                let temp = (lhs as $uD).wrapping_mul(rhs as $uD);
                (temp as $uX, (temp >> ($n_h * 2)) as $uX)
            }
            #[inline(always)]
            pub fn carrying_mul_add(lhs: $uX, mul: $uX, add: $uX) -> ($uX, $uX) {
                let temp = (lhs as $uD).wrapping_mul(mul as $uD).wrapping_add(add as $uD);
                (temp as $uX, (temp >> ($n_h * 2)) as $uX)
            }
            //the number of bits in a $uX
            let n = $n_h * 2;
            //the number of bits in a $uD
            let d_n = n * 2;
            //`lo` and `hi` refer to the high and low `n` bits of a `$uX`,
            //`0` - `3` refer to the 4 `n_h` divisions of a `$uD`,
            //and `mid` refer to the middle two `n_h` divisions.
            let div_leading_zeros = div.leading_zeros();
            //these first two branches are done for both optimization, and because
            //the third branch breaks down if `div_leading_zeros` is not in the mid range
            if div_leading_zeros >= (n + $n_h) {
                //`div` has `n_h` significant bits or less.
                if div_leading_zeros == d_n {
                    panic!("division by zero")
                }
                //long division algorithm.
                //this is optimized for assuming `duo` has few leading zeros so no branching here.
                let div_lo = (div as $uH) as $uX;

                let duo_hi = (duo >> n) as $uX;
                let quo_hi = duo_hi.wrapping_div(div_lo);
                //all of the `rem_{}`s cannot be larger than `$uH::MAX`,
                //since `div` cannot be larger than `$uH::MAX`.
                let rem_2 = duo_hi.wrapping_rem(div_lo) as $uH;

                let duo_mid =
                    ((rem_2 as $uX) << $n_h) |
                    (((duo >> $n_h) as $uH) as $uX);
                let quo_1 = duo_mid.wrapping_div(div_lo) as $uH;
                let rem_1 = duo_mid.wrapping_rem(div_lo) as $uH;

                let duo_lo =
                    ((rem_1 as $uX) << $n_h) |
                    ((duo as $uH) as $uX);
                let quo_0 = duo_lo.wrapping_div(div_lo) as $uH;
                (
                    ((quo_hi as $uD) << n) | ((quo_1 as $uD) << $n_h) | (quo_0 as $uD),
                    (duo_lo.wrapping_rem(div_lo) as $uH) as $uD
                )
            } else if div_leading_zeros < $n_h {
                let duo_leading_zeros = duo.leading_zeros();
                if duo_leading_zeros >= div_leading_zeros {
                    if duo >= div {
                        //duo cannot be twice div or more if their significant bit places
                        //are at least the same.
                        return (1,duo.wrapping_sub(div))
                    } else {
                        return (0,duo)
                    }
                }

                //Proof that `quo` can only be `mult` or `mult - 1`.
                //disclaimer: this is not rigorous, and some parts are handwaved
                //We are trying to find the quotient, `quo`.
                //1. quo = duo / div.
                //just the higher half of the bits.
                //2. duo_hi = duo / 2^n.
                //3. div_hi = div / 2^n.
                //4. mult = duo_hi / div_hi.
                //this is due to the `div_leading_zeros < n_h` branch above.
                //5. 2^(n_h + n) <= div < 2^d_n.
                //this is due to the `duo_leading_zeros >= div_leading_zeros` branch above.
                //6. 2 * div <= duo < 2^d_n.
                //7. 2^n_h <= div_hi < 2^n.
                //8. 2 * div_hi <= duo_hi < 2^n.
                //9. duo_hi / div_hi >= 2.
                //10. (duo_hi * 2^n) <= duo < ((duo_hi + 1) * 2^n).
                //11. (duo_hi * 2^n) / div <= duo / div < ((duo_hi + 1) * 2^n) / div.
                //12. (div_hi * 2^n) <= div < (div_hi + 1) * 2^n.
                //Statement x has the bounds on `duo / div` (which is `quo`), but because we want `quo` in
                //terms of `duo_hi` and `div_hi`, we substitute the bounds of `div` in terms of `div_hi`
                //(#12) for `div`.
                //13. {(duo_hi * 2^n) / (div_hi * 2^n) , (duo_hi * 2^n) / ((div_hi + 1) * 2^n)} <= quo <
                //{((duo_hi + 1) * 2^n) / (div_hi * 2^n), ((duo_hi + 1) * 2^n) / ((div_hi + 1) * 2^n)}.
                //finding the widest bounds of `quo` and cancelling the `2^n`.
                //14. duo_hi / (div_hi + 1) <= quo < (duo_hi + 1) / div_hi
                //15. `(duo_hi + 1) / div_hi` can only be `mult` or `mult + 1`
                //16. because `div_hi` has to be at least `2^n_h` (#7), and `duo_hi` cannot be more than
                //`2^n - 1` (#8), `duo_hi / (div_hi + 1)` can only be `mult - 1` or `mult`
                //17. mult - 1 <= quo < mult + 1

                //Assuming first that `quo` is `mult - 1`, we can deduct by the definition of the quotient that
                //`quo * (mult - 1) <= duo`, and also that `quo * mult > duo`.
                //if `quo * mult <= duo`, this assumption is false and there is only one other possibility
                //for what `quo` is, `mult`.
                //Thus, we find the quotient using only an `n` sized divide to find `mult`
                //and a `n` by `d_n` sized multiply and comparison to find if `quo * mult > duo`.
                let duo_hi = (duo >> n) as $uX;
                let div_hi = (div >> n) as $uX;
                let mult = duo_hi.wrapping_div(div_hi);
                //inline `n` bit by `d_n` bit multiplication and overflow check
                //(`mult` can have `n_h` + 1 significant bits, and we cannot do `mult * div > duo` directly
                //because of possibility of overflow)
                let div_lo = div as $uX;
                let (temp_lo,carry) = carrying_mul(mult,div_lo);
                let (temp_hi,overflow) = carrying_mul_add(mult,div_hi,carry);
                if (overflow != 0) || (((temp_lo as $uD) | ((temp_hi as $uD) << n)) > duo) {
                    //duo cannot be more than `2^d_n - 1`, and overflow means a value more than that
                    let (temp_lo,carry) = carrying_mul(mult.wrapping_sub(1),div_lo);
                    let temp_hi = mult.wrapping_sub(1).wrapping_mul(div_hi).wrapping_add(carry);
                    return (
                        mult.wrapping_sub(1) as $uD,
                        duo.wrapping_sub((temp_lo as $uD) | ((temp_hi as $uD) << n))
                    )
                } else {
                    return (
                        mult as $uD,
                        duo.wrapping_sub((temp_lo as $uD) | ((temp_hi as $uD) << n))
                    )
                }
            } else {
                let mut duo_leading_zeros = duo.leading_zeros();
                if duo_leading_zeros >= div_leading_zeros {
                    if duo >= div {
                        return (1,duo.wrapping_sub(div))
                    } else {
                        return (0,duo)
                    }
                }
                if duo_leading_zeros >= n {
                    //`duo < 2^n` so it will fit in a $uX.
                    //`div` will also fit in a $uX (because of the `duo_leading_zeros >= div_leading_zeros`
                    //branch) so no numerical error.
                    return (
                        (duo as $uX).wrapping_div(div as $uX) as $uD,
                        (duo as $uX).wrapping_rem(div as $uX) as $uD
                    )
                }
                //Instead of clearing an average of 1.5 bits from `duo` per iteration via
                //binary long division (assuming that the bits are random), an average of
                //`n_h - 0.5` bits are cleared per iteration with this algorithm.
                //It is a more complicated version of long division.
                //For an example, consider the division of 76543210 by 213 and assume that `n_h`
                //is equal to two decimal digits (note: since 10 is not a power of 2, this algorithm
                //might not work for other cases). The first `n_h` part of the divisor (21) is taken
                //and is incremented by 1 to prevent oversubtraction.
                //in the first step, the first `n` part of duo (7654) is divided by the 22 to make 347.
                //We remember that there was one extra place not in the `n_h` part of the divisor and
                //shift the 347 right by one, in contrast to a normal long division. The 347 is
                //multiplied by the whole divisor to make 73911, and subtracted from duo to finish the
                //step.
                //    347
                //  ________
                // |76543210
                // -73911
                //   2632210
                //two more steps are taken after this and then duo fits into an `uX`, and then a final
                //normal long division step is made
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
                //the tower at the top is added together to produce the quotient, 359357 (but in the
                //actual algorithm, the quotient is progressively added to each step instead of at
                //the end).
                //`duo` is called `duo` because it acts both as the dividend and eventually becomes the remainder
                let mut duo = duo;
                //the number of lesser significant bits not a part of `div_sig_n_h`
                let div_lesser_places = (n + $n_h).wrapping_sub(div_leading_zeros);
                //the most significant `n_h` bits of div
                let div_sig_n_h = (div >> div_lesser_places) as $uH;
                //has to be a `$uX` in case of overflow
                let div_sig_n_h_add1 = (div_sig_n_h as $uX).wrapping_add(1);
                let mut quo: $uD = 0;
                loop {
                    let duo_lesser_places = n.wrapping_sub(duo_leading_zeros);
                    let duo_sig_n = (duo >> duo_lesser_places) as $uX;
                    let mult = duo_sig_n.wrapping_div(div_sig_n_h_add1) as $uD;
                    if duo_lesser_places > div_lesser_places {
                        let place = duo_lesser_places.wrapping_sub(div_lesser_places);
                        //addition to the quotient
                        quo = quo.wrapping_add(mult << place);
                        let temp = div.wrapping_mul(mult);
                        //subtraction from `duo`
                        //at least `n_h - 1` bits are cleared from `duo` here
                        duo = duo.wrapping_sub(temp << place);
                    } else {
                        //if the divisor is large enough, the shift will actually be right
                        let place = div_lesser_places.wrapping_sub(duo_lesser_places);
                        let temp = (mult >> place) as $uX;
                        quo = quo.wrapping_add(temp as $uD);
                        //inline multiplication of a 64 bit integer by a 128 bit integer.
                        //overflow discarded because it is not possible.
                        let div_lo = div as $uX;
                        let div_hi = (div >> n) as $uX;
                        let (temp_lo,carry) = carrying_mul(temp,div_lo);
                        let temp_hi = temp.wrapping_mul(div_hi).wrapping_add(carry);
                        //at least `n_h - 1` bits are cleared from `duo` here
                        duo = duo.wrapping_sub((temp_lo as $uD) | ((temp_hi as $uD) << n));
                    }
                    duo_leading_zeros = duo.leading_zeros();
                    if duo_leading_zeros >= div_leading_zeros {
                        if duo >= div {
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
                    if duo_leading_zeros >= n {
                        //`div_leading_zeros` is more than `n` (or else it is caught by
                        //the `duo_leading_zeros >= div_leading_zeros` branch) so no numerical error
                        return (
                            quo.wrapping_add((duo as $uX).wrapping_div(div as $uX) as $uD),
                            (duo as $uX).wrapping_rem(div as $uX) as $uD
                        )
                    }
                }
            }
        }

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
            //checks all possible single continuous strings of ones (except for division by zero)
            let mut lhs0: T = 0;
            for i0 in 0..n {
                lhs0 <<= 1;
                lhs0 |= 1;
                let mut lhs1 = lhs0;
                for i1 in 0..i0 {
                    lhs1 ^= 1 << i1;
                    let mut rhs0: T = 0;
                    for i2 in 0..n {
                        rhs0 <<= 1;
                        rhs0 |= 1;
                        let mut rhs1 = rhs0;
                        for i3 in 0..i2 {
                            rhs1 ^= 1 << i3;
                            if rhs1 == 0 {
                                continue
                            }
                            assert_eq!((lhs1 / rhs1, lhs1 % rhs1),$unsigned_name(lhs1,rhs1));
                            assert_eq!(((lhs1 as $iD) / (rhs1 as $iD), (lhs1 as $iD) % (rhs1 as $iD)),$signed_name(lhs1 as $iD,rhs1 as $iD));
                        }
                    }
                }
            }
        }
    }
}

impl_div_rem!(u32_div_rem, i32_div_rem, u32_i32_div_rem_test, 8u32, u8, u16, u32, i32, inline, doc = "Computes the quotient and remainder of `duo` divided by `rem` and returns them as a tuple."; inline, doc = "Computes the quotient and remainder of `duo` divided by `rem` and returns them as a tuple.");
impl_div_rem!(u64_div_rem, i64_div_rem, u64_i64_div_rem_test, 16u32, u16, u32, u64, i64, inline, doc = "Computes the quotient and remainder of `duo` divided by `rem` and returns them as a tuple."; inline, doc = "Computes the quotient and remainder of `duo` divided by `rem` and returns them as a tuple.");
impl_div_rem!(u128_div_rem, i128_div_rem, u128_i128_div_rem_test, 32u32, u32, u64, u128, i128, inline, doc = "Computes the quotient and remainder of `duo` divided by `rem` and returns them as a tuple."; inline, doc = "Computes the quotient and remainder of `duo` divided by `rem` and returns them as a tuple.");