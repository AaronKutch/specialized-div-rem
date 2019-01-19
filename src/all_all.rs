/// Generates a function that returns the quotient and remainder of unsigned integer division of
/// `duo` by `div`. The function uses 3 different algorithms (and several conditionals for simple
/// cases) that handle almost all numerical magnitudes efficiently.
macro_rules! impl_div_rem {
    (
        $unsigned_name:ident, //name of the unsigned function
        $signed_name:ident, //name of the signed function
        $test_name:ident, //name of the test function
        $n_h:expr, //the number of bits in $iH or $uH
        $uH:ident, //unsigned integer with half the bit width of $uX
        $uX:ident, //the largest division instruction that this function calls operates on this
        $uD:ident, //unsigned integer with double the bit width of $uX
        $iD:ident, //signed version of $uD
        $bit_selector_max:expr, //the max value of the smallest bit string needed to index the bits of an $uD
        $($unsigned_attr:meta),*; //attributes for the unsigned function
        $($signed_attr:meta),* //attributes for the signed function
    ) => {
        //wrapping_{} was used everywhere for better performance when compiling with
        //`debug-assertions = true`

        /// Computes the quotient and remainder of `duo` divided by `div` and returns them as a
        /// tuple.
        /// 
        /// # Panics
        /// 
        /// When attempting to divide by zero, this function will panic.
        $(
            #[$unsigned_attr]
        )*
        pub fn $unsigned_name(duo: $uD, div: $uD) -> ($uD,$uD) {
            //TODO if and when `carrying_mul` (rust-lang rfc #2417) is stabilized, this can be fixed
            #[inline(always)]
            pub fn carrying_mul(lhs: $uX, rhs: $uX) -> ($uX, $uX) {
                let tmp = (lhs as $uD).wrapping_mul(rhs as $uD);
                (tmp as $uX, (tmp >> ($n_h * 2)) as $uX)
            }
            #[inline(always)]
            pub fn carrying_mul_add(lhs: $uX, mul: $uX, add: $uX) -> ($uX, $uX) {
                let tmp = (lhs as $uD).wrapping_mul(mul as $uD).wrapping_add(add as $uD);
                (tmp as $uX, (tmp >> ($n_h * 2)) as $uX)
            }
            //the number of bits in a $uX
            let n = $n_h * 2;
            //the number of bits in a $uD
            let n_d = n * 2;
            //`lo` and `hi` refer to the high and low `n` bits of a `$uX`,
            //`0` to `3` refer to the 4 `n_h` bit parts of a `$uD`,
            //and `mid` refer to the middle two `n_h` parts.
            let div_lz = div.leading_zeros();
            //the possible ranges `duo` and `div` are at this point
            //`0 <= duo < 2^n_d`
            //`0 <= div < 2^n_d`
            //division by zero check
            if div_lz == n_d {
                panic!("division by zero")
            }
            let mut duo_lz = duo.leading_zeros();
            //`0 <= duo < 2^n_d`
            //`1 <= div < 2^n_d`
            //quotient is 0 or 1 check
            if div_lz <= duo_lz {
                //the quotient cannot be more than 1.
                //Visually, when the most significant bit is at the 8th place, the closest that the
                //quotient can come to 2 is 11111111 / 10000000 = ~1.992 in decimal. Longer strings
                //approach but never reach 2.
                if duo >= div {
                    return (1,duo.wrapping_sub(div))
                } else {
                    return (0,duo)
                }
            }
            //`_sb` is the number of significant bits (from the lsb to the last 1 bit)
            //`{2, 2^div_sb} <= duo < 2^n_d`
            //`1 <= div < {2^duo_sb, 2^(n_d - 1)}`
            //smaller division check
            if duo_lz >= n {
                //`duo < 2^n` so it will fit in a $uX.
                //`div` will also fit in a $uX (because of the
                //`div_lz <= duo_lz` branch) so no numerical error.
                return (
                    (duo as $uX).wrapping_div(div as $uX) as $uD,
                    (duo as $uX).wrapping_rem(div as $uX) as $uD
                )
            }
            //`{2^n, 2^div_sb} <= duo < 2^n_d`
            //`1 <= div < {2^duo_sb, 2^(n_d - 1)}`
            //regular long division algorithm
            if div_lz >= (n + $n_h) {
                //this is optimized for assuming `duo` has few leading zeros so no branching here.
                let div_lo = (div as $uH) as $uX;

                let duo_hi = (duo >> n) as $uX;
                let quo_hi = duo_hi.wrapping_div(div_lo);
                //all of the `rem_{}`s cannot be larger than `$uH::MAX`,
                //since `div` cannot be larger than `$uH::MAX`.
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
            }
            //`{2^n, 2^div_sb} <= duo < 2^n_d`
            //`2^n_h <= div < {2^duo_sb, 2^(n_d - 1)}`
            //relative leading significant bits, cannot be negative
            let rel_leading_sb = div_lz.wrapping_sub(duo_lz);
            //`mul` or `mul - 1` algorithm
            if rel_leading_sb < $n_h {
                //TODO: fix this
                //unfinished proof that `quo` can only be `mul` or `mul - 1`.
                //disclaimer: this is not rigorous, and some parts are handwaved
                //We are trying to find the quotient, `quo`.
                //1. quo = duo / div. (definition)
                //`shift` is the number of bits not in the higher `n` significant bits of `duo`
                //2. shift = n - duo_lz. (definition)
                //3. duo_sig_n = duo / 2^shift. (definition)
                //4. div_sig_n = div / 2^shift. (definition)
                //this is because of the bits less significant than the sig_n bits that are cut off
                //during the bit shift
                //5. duo_sig_n * 2^shift <= duo < (duo_sig_n + 1) * 2^shift.
                //6. div_sig_n * 2^shift <= div < (div_sig_n + 1) * 2^shift.
                //dividing each bound of 5. by each bound of 6.
                //(duo_sig_n * 2^shift) / (div_sig_n * 2^shift)
                //(duo_sig_n * 2^shift) / ((div_sig_n + 1) * 2^shift)
                //((duo_sig_n + 1) * 2^shift) / (div_sig_n * 2^shift)
                //((duo_sig_n + 1) * 2^shift) / ((div_sig_n + 1) * 2^shift)
                //simplifying each of these four
                //duo_sig_n / div_sig_n
                //duo_sig_n / (div_sig_n + 1)
                //(duo_sig_n + 1) / div_sig_n
                //(duo_sig_n + 1) / (div_sig_n + 1)
                //taking the smallest and the largest of these as the low and high bounds
                //and replacing `duo / div` with `quo`
                //7. duo_sig_n / (div_sig_n + 1) <= quo < (duo_sig_n + 1) / div_sig_n
                //8. mul = duo_sig_n / div_sig_n. (definition)
                //because of the range restraints on `duo_sig_n` and `div_sig_n` leading up to this,
                //9. `duo_sig_n / (div_sig_n + 1)` can only be `mul` or `mul - 1`
                //10. `(duo_sig_n + 1) / div_sig_n` can only be `mul` or `mul + 1`
                //11.  mul - 1 <= quo < mul + 1
                //Thus, we find the quotient using only an `n` sized divide to find `mul`
                //and a `n` by `d_n` sized multiply and comparison to find if `quo * mul > duo`
                let shift = n.wrapping_sub(duo_lz);
                let duo_sig_n = (duo >> shift) as $uX;
                let div_sig_n = (div >> shift) as $uX;
                let mul = duo_sig_n.wrapping_div(div_sig_n);
                //inline `n` bit by `n_d` bit multiplication and overflow check (we cannot do
                //`mul * div > duo` directly because of possibility of overflow)
                //duo cannot be more than `2^n_d - 1`, and overflow means a value more than that
                let div_lo = div as $uX;
                let div_hi = (div >> n) as $uX;
                let (tmp_lo,carry) = carrying_mul(mul,div_lo);
                let (tmp_hi,overflow) = carrying_mul_add(mul,div_hi,carry);
                if (overflow != 0) || (((tmp_lo as $uD) | ((tmp_hi as $uD) << n)) > duo) {
                    //The only problem now is to find the remainder
                    //Setting `div*mul + x` equal to `div*(mul - 1)` and doing algebra, we arrive at
                    //x = -div.
                    //We can avoid doing the `div*(mul - 1)` calculation by just subtracting `div`
                    //from the `div*mul` that is already calculated
                    //Note that the overflow cannot be more than 1, otherwise the `div` subtraction
                    //would not be able to bring the remainder value below 2^n_d - 1, which
                    //contradicts many things.
                    //the remainder will be `duo - (div*mul - div)` which can be simplified
                    return (
                        mul.wrapping_sub(1) as $uD,
                        duo.wrapping_add(div.wrapping_sub((tmp_lo as $uD) | ((tmp_hi as $uD) << n)))
                    )
                } else {
                    return (
                        mul as $uD,
                        duo.wrapping_sub((tmp_lo as $uD) | ((tmp_hi as $uD) << n))
                    )
                }
            }
            //special long division algorithm. This algorithm only works with a very restricted
            //subset of the possible values of `duo` and `div`, hence why many special cases were
            //tested above
            //Instead of clearing a minimum of 1 bit from `duo` per iteration via
            //binary long division, `n_h - 1` bits are cleared per iteration with this algorithm.
            //It is a more complicated version of long division.
            //For an example, consider the division of 76543210 by 213 and assume that `n_h`
            //is equal to two decimal digits (note: we are working with base 10 here for
            //readability. I don't know if the algorithm breaks in general for non powers of two).
            //The first `h_n` part of the divisor (21) is taken and is incremented by
            //1 to prevent oversubtraction.
            //in the first step, the first `n` part of duo (7654) is divided by the 22 to make 347.
            //We remember that there was 1 extra place not in the `n_h` part of the divisor and
            //shift the 347 right by 1, in contrast to a normal long division. The 347 is
            //multiplied by the whole divisor to make 73911, and subtracted from duo to finish the
            //step.
            //    347
            //  ________
            // |76543210
            // -73911
            //   2632210
            //two more steps are taken after this and then duo fits into `n` bits, and then a final
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
            //In the actual algorithm below, instead of the final normal long division step, one of
            //the three other algorithms ("quotient is 0 or 1", "mul or mul - 1", "n sized division"
            //) is used.
            let mut duo = duo;
            //the number of lesser significant bits not a part of `div_sig_n_h`. Has to be positive.
            let div_lesser_places = (n + $n_h).wrapping_sub(div_lz);
            //the most significant `n_h` bits of div
            let div_sig_n_h = (div >> div_lesser_places) as $uH;
            //has to be a `$uX` in case of overflow
            let div_sig_n_h_add1 = (div_sig_n_h as $uX).wrapping_add(1);
            let mut quo: $uD = 0;
            //The two main loops here are the same, but the third loop `return`s are different for
            //the two branches.
            if div_lz >= n {
                //`{2^n, 2^(div_sb + n_h)} <= duo < 2^n_d`
                //`2^n_h <= div < {2^(duo_sb - n_h), 2^n}`
                loop {
                    let duo_lesser_places = n.wrapping_sub(duo_lz);
                    let duo_sig_n = (duo >> duo_lesser_places) as $uX;
                    let mul = duo_sig_n.wrapping_div(div_sig_n_h_add1) as $uD;
                    if div_lesser_places <= duo_lesser_places {
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
                        let mul = duo_sig_n.wrapping_div(div_sig_n);
                        let div_lo = div as $uX;
                        let div_hi = (div >> n) as $uX;
                        let (tmp_lo,carry) = carrying_mul(mul,div_lo);
                        let (tmp_hi,overflow) = carrying_mul_add(mul,div_hi,carry);
                        if (overflow != 0) || (((tmp_lo as $uD) | ((tmp_hi as $uD) << n)) > duo) {
                            return (
                                //note that this time, the "mul or mul - 1" result is added to `quo`
                                //to get the final correct quotient
                                quo.wrapping_add(mul.wrapping_sub(1) as $uD),
                                duo.wrapping_add(
                                    div.wrapping_sub((tmp_lo as $uD) | ((tmp_hi as $uD) << n))
                                )
                            )
                        } else {
                            return (
                                quo.wrapping_add(mul as $uD),
                                duo.wrapping_sub((tmp_lo as $uD) | ((tmp_hi as $uD) << n))
                            )
                        }
                    }
                    duo_lz = duo.leading_zeros();
                    if duo_lz >= n {
                        //simple division and addition
                        return (
                            quo.wrapping_add((duo as $uX).wrapping_div(div as $uX) as $uD),
                            (duo as $uX).wrapping_rem(div as $uX) as $uD
                        )
                    }
                }
            } else {
                //`{2^n, 2^(div_sb + n_h)} <= duo < 2^n_d`
                //`2^n <= div < {2^(duo_sb - n_h), 2^(n + n_h - 1)}`
                loop {
                    let duo_lesser_places = n.wrapping_sub(duo_lz);
                    let duo_sig_n = (duo >> duo_lesser_places) as $uX;
                    let mul = duo_sig_n.wrapping_div(div_sig_n_h_add1) as $uD;
                    if div_lesser_places <= duo_lesser_places {
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
                        let mul = duo_sig_n.wrapping_div(div_sig_n);
                        let div_lo = div as $uX;
                        let div_hi = (div >> n) as $uX;
                        let (tmp_lo,carry) = carrying_mul(mul,div_lo);
                        let (tmp_hi,overflow) = carrying_mul_add(mul,div_hi,carry);
                        if (overflow != 0) || (((tmp_lo as $uD) | ((tmp_hi as $uD) << n)) > duo) {
                            return (
                                quo.wrapping_add(mul.wrapping_sub(1) as $uD),
                                duo.wrapping_add(
                                    div.wrapping_sub((tmp_lo as $uD) | ((tmp_hi as $uD) << n))
                                )
                            )
                        } else {
                            return (
                                quo.wrapping_add(mul as $uD),
                                duo.wrapping_sub((tmp_lo as $uD) | ((tmp_hi as $uD) << n))
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
                }
            }
        }

        /// Computes the quotient and remainder of `duo` divided by `div` and returns them as a
        /// tuple.
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
            //checks all possible single continuous strings of ones (except when all bits are zero)
            //uses about 68 million iterations for T = u128
            let mut lhs0: T = 1;
            for i0 in 1..=n {
                let mut lhs1 = lhs0;
                for i1 in 0..i0 {
                    let mut rhs0: T = 1;
                    for i2 in 1..=n {
                        let mut rhs1 = rhs0;
                        for i3 in 0..i2 {
                            assert_eq!((lhs1 / rhs1, lhs1 % rhs1),$unsigned_name(lhs1,rhs1));
                            //avoid the $iD::MIN/-1 overflow
                            if !((lhs1 as $iD == ::core::$iD::MIN) && (rhs1 as $iD == -1)) {
                                assert_eq!(
                                    ((lhs1 as $iD) / (rhs1 as $iD), (lhs1 as $iD) % (rhs1 as $iD)),
                                    $signed_name(lhs1 as $iD,rhs1 as $iD)
                                );
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
            //binary fuzzer
            use rand::random;
            let mut lhs: T = 0;
            let mut rhs: T = 0;
            let mut ones: T;
            for _ in 0..10_000_000 {
                let r0: u32 = $bit_selector_max & random::<u32>();
                ones = 0;
                for _ in 0..r0 {
                    ones <<= 1;
                    ones |= 1;
                }
                let r1: u32 = $bit_selector_max & random::<u32>();
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
                    assert_eq!((lhs / rhs, lhs % rhs),$unsigned_name(lhs,rhs));
                    if !((lhs as $iD == ::core::$iD::MIN) && (rhs as $iD == -1)) {
                        assert_eq!(
                            ((lhs as $iD) / (rhs as $iD), (lhs as $iD) % (rhs as $iD)),
                            $signed_name(lhs as $iD,rhs as $iD)
                        );
                    }
                }
            }
            assert_eq!($signed_name(::core::$iD::MIN,-1),(::core::$iD::MIN,0));
        }
    }
}

impl_div_rem!(u32_div_rem, i32_div_rem, u32_i32_div_rem_test, 8u32, u8, u16, u32, i32, 0b11111u32, inline; inline, doc = "Note that unlike some of Rust's division functions, `i32_div_rem(i32::MIN,-1)` will not panic but instead overflow and produce the correct truncated two's complement `(i32::MIN,0)`.");
impl_div_rem!(u64_div_rem, i64_div_rem, u64_i64_div_rem_test, 16u32, u16, u32, u64, i64, 0b111111u32, inline; inline, doc = "Note that unlike some of Rust's division functions, `i64_div_rem(i64::MIN,-1)` will not panic but instead overflow and produce the correct truncated two's complement `(i64::MIN,0)`.");
impl_div_rem!(u128_div_rem, i128_div_rem, u128_i128_div_rem_test, 32u32, u32, u64, u128, i128, 0b1111111u32, inline; inline, doc = "Note that unlike some of Rust's division functions, `i128_div_rem(i128::MIN,-1)` will not panic but instead overflow and produce the correct truncated two's complement `(i128::MIN,0)`.");