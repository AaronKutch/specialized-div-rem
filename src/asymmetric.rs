macro_rules! impl_asymmetric {
    (
        $unsigned_name:ident, // name of the unsigned function
        $signed_name:ident, // name of the signed function
        $test_name:ident, // name of the test function
        $n_d_by_n_division:ident, // function for division of a $uD by a $uX
        $n_h:expr, // the number of bits in $iH or $uH
        $uH:ident, // unsigned integer with half the bit width of $uX
        $uX:ident, // unsigned integer with half the bit width of $uD
        $uD:ident, // unsigned integer with double the bit width of $uX
        $iD:ident, // signed version of $uD
        $($unsigned_attr:meta),*; // attributes for the unsigned function
        $($signed_attr:meta),* // attributes for the signed function
    ) => {
        /// Computes the quotient and remainder of `duo` divided by `div` and returns them as a
        /// tuple. This is optimized for dividing integers with the same bitwidth as the largest
        /// double register operand in an asymmetric division such as the x86-64 `divq` assembly
        /// instruction which can divide a 128 bit integer by a 64 bit integer if the quotient fits
        /// in 64 bits.
        ///
        /// # Panics
        ///
        /// When attempting to divide by zero, this function will panic.
        $(
            #[$unsigned_attr]
        )*
        pub fn $unsigned_name(duo: $uD, div: $uD) -> ($uD,$uD) {
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

            let n: u32 = $n_h * 2;

            // Many of these subalgorithms are taken from trifecta.rs, see that for better
            // documentation

            let duo_lo = duo as $uX;
            let duo_hi = (duo >> n) as $uX;
            let div_lo = div as $uX;
            let div_hi = (div >> n) as $uX;
            if div_hi == 0 {
                if div_lo == 0 {
                    panic!("division by zero");
                }
                if duo_hi < div_lo {
                    // plain $uD by $uX division that will fit into $uX
                    let tmp = unsafe { $n_d_by_n_division(duo, div_lo) };
                    return (tmp.0 as $uD, tmp.1 as $uD)
                } else if (div_lo >> $n_h) == 0 {
                    // Short division of $uD by a $uH.
                    let div_lo = (div as $uH) as $uX;
                    let duo_hi = (duo >> n) as $uX;
                    let quo_hi = duo_hi.wrapping_div(div_lo);
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
                    // Short division using the $uD by $uX division
                    let (quo_hi, rem_hi) = (duo_hi / div_lo, duo_hi % div_lo);
                    let tmp = unsafe {
                        $n_d_by_n_division((duo_lo as $uD) | ((rem_hi as $uD) << n), div_lo)
                    };
                    return ((tmp.0 as $uD) | ((quo_hi as $uD) << n), tmp.1 as $uD)
                }
            }

            let duo_lz = duo_hi.leading_zeros();
            let div_lz = div_hi.leading_zeros();
            let rel_leading_sb = div_lz.wrapping_sub(duo_lz);
            if rel_leading_sb < $n_h {
                // Some x86_64 CPUs have bad `divq` implementations that make putting
                // a `mul` or `mul - 1` algorithm here beneficial
                let shift = n.wrapping_sub(duo_lz);
                let duo_sig_n = (duo >> shift) as $uX;
                let div_sig_n = (div >> shift) as $uX;
                let mul = duo_sig_n.wrapping_div(div_sig_n);
                let div_lo = div as $uX;
                let div_hi = (div >> n) as $uX;
                let (tmp_lo, carry) = carrying_mul(mul,div_lo);
                let (tmp_hi, overflow) = carrying_mul_add(mul,div_hi,carry);
                let tmp = (tmp_lo as $uD) | ((tmp_hi as $uD) << n);
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
            } else {
                // This has been adapted from
                // https://www.codeproject.com/tips/785014/uint-division-modulus which was in turn
                // adapted from www.hackersdelight.org

                // This is similar to the `mul` or `mul - 1` algorithm in that it uses only more
                // significant parts of `duo` and `div` to divide a large integer with a smaller
                // division instruction.
                let tmp = unsafe {
                    $n_d_by_n_division(duo >> 1, ((div << div_lz) >> n) as $uX)
                };
                let mut quo = tmp.0 >> ((n - 1) - div_lz);
                if quo != 0 {
                    quo -= 1;
                }
                // Note that this is a large $uD multiplication being used here
                let mut rem = duo - ((quo as $uD) * div);
    
                if rem >= div {
                    quo += 1;
                    rem -= div;
                }
                return (quo as $uD, rem)
            }
        }

        /// Computes the quotient and remainder of `duo` divided by `div` and returns them as a
        /// tuple. This is optimized for dividing integers with the same bitwidth as the largest
        /// double register operand in an asymmetric division such as the x86-64 `divq` assembly
        /// instruction which can divide a 128 bit integer by a 64 bit integer if the quotient fits
        /// in 64 bits.
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
