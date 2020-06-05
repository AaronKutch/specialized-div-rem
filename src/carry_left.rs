macro_rules! impl_carry_left {
    (
        $unsigned_name:ident, // name of the unsigned division function
        $signed_name:ident, // name of the signed division function
        $n:expr, // the number of bits in a $iX or $uX
        $uX:ident, // unsigned integer type for the inputs and outputs of `$unsigned_name`
        $iX:ident, // signed integer type for the inputs and outputs of `$signed_name`
        $($unsigned_attr:meta),*; // attributes for the unsigned function
        $($signed_attr:meta),* // attributes for the signed function
    ) => {
        /// Computes the quotient and remainder of `duo` divided by `div` and returns them as a
        /// tuple.
        ///
        /// 
        ///
        /// # Panics
        ///
        /// When attempting to divide by zero, this function will panic.
        $(
            #[$unsigned_attr]
        )*
        pub fn $unsigned_name(duo: $uX, div: $uX) -> ($uX, $uX) {
            let duo_original = duo;
            if div == 0 {
                panic!("division by zero")
            }
            if duo < div {
                return (0, duo)
            }
            // This eliminates cases where the most significant bit of `div` is set. Signed
            // comparisons can then be used to determine overflow without needing carry flags.
            if (duo - div) < div {
                return (1, duo - div)
            }
            
            let mut level = $n / 4;
            let mut shift = $n / 2;
            loop {
                let sub = (duo >> shift).wrapping_sub(div);
                if (sub as $iX) < 0 {
                    // shift is too high
                    shift -= level;
                } else {
                    // shift is too low
                    shift += level;
                }
                if level == 1 {
                    // final step
                    let sub = (duo >> shift).wrapping_sub(div);
                    if (sub as $iX) < 0 {
                        shift -= 1;
                        break
                    } else {
                        break
                    }
                }
                // narrow down bisection
                level >>= 1;
            }
            
            // If the architecture has flags and predicated arithmetic instructions, it is possible
            // to do binary long division without branching. This is adapted from 
            // http://www.chiark.greenend.org.uk/~theom/riscos/docs/ultimate/a252div.txt
            //
            // What allows doing division in only 3 instructions is realizing that instead of
            // keeping `duo` in place and shifting `div` right to align bits, `div` can be kept in
            // place and `duo` can be shifted left. This would not normally save any instructions
            // and just cause more edge case problems and make `duo < div_original` tests harder.
            // However, some architectures have an option to shift an argument in an arithmetic
            // operation, meaning `duo` can be shifted left and subtracted from in one instruction.
            // `div` never has to be written to in the loop. The other two instructions are updating
            // `quo` and undoing the subtraction if it turns out things were not normalized.

            // Perform one binary long division step on the already normalized arguments, and setup
            // all the variables.
            /*
            let div_original = div;
            let mut div: $uX = (div << shift);
            let mut quo: $uX = 1;
            let mut duo = duo.wrapping_sub(div);
            if duo < div_original {
                // early return for powers of two and close to powers of two
                return (1 << shift, duo);
            }
            // The add-with-carry that updates `quo` needs to have the carry set when a normalized
            // subtract happens. Using `duo.wrapping_shl(1).overflowing_sub(div)` to do the
            // subtraction generates a carry when an unnormalized subtract happens, which is the
            // opposite of what we want. Instead, we use
            // `duo.wrapping_shl(1).overflowing_add(div_neg)`, where `div_neg` is negative `div`.
            let div_neg: $uX;
            if div >= (1 << ($n - 1)) {
                // A very ugly edge case where the most significant bit of `div` is set (after
                // shifting to match `duo` when its most significant bit is at the sign bit), which
                // leads to the sign bit of `div_neg` being cut off and carries not happening when
                // they should. This performs a long division step that keeps `duo` in place and
                // shifts `div` down.
                div >>= 1;
                div_neg = div.wrapping_neg();
                let (sub, carry) = duo.overflowing_add(div_neg);
                duo = sub;
                quo = quo.wrapping_add(quo).wrapping_add(carry as $uX);
                if !carry {
                    duo = duo.wrapping_add(div);
                }
                shift -= 1;
            } else {
                div_neg = div.wrapping_neg();
            }
            for _ in 0..shift {
                // `ADDS duo, div, duo, LSL #1`
                // (add div to duo shifted left 1 and set flags)
                let (sub, carry) = duo.wrapping_shl(1).overflowing_add(div_neg);
                duo = sub;
                // `ADC quo, quo, quo`
                // (add with carry). Effectively shifts `quo` left by 1 and sets the least
                // significant bit to the carry.
                quo = quo.wrapping_add(quo).wrapping_add(carry as $uX);
                // `ADDCC duo, duo, div`
                // (add if carry clear). Undoes the subtraction if no carry was generated.
                if !carry {
                    // undo the subtraction
                    duo = duo.wrapping_add(div);
                }
            }
            (quo, duo >> shift)
            */
            
            println!("duo:{:08b}, div:{:08b}", duo, div);

            // first division step
            let div_original = div;
            let mut div: $uX = (div << shift);
            let mut duo = duo.wrapping_sub(div);
            if duo < div_original {
                return (1 << shift, duo);
            }
            println!("duo:{:08b}, div:{:08b}", duo, div);

            let div_neg_add_1: $uX;
            let div_sub_1: $uX;

            // this contains quotient bits that are not added on until the end
            let mut quo: $uX = 1 << shift;
            let mask: $uX;
            if div >= (1 << ($n - 1)) {
                // deal with same edge case as before, but the quotient bit from this step has to be
                // stored in `quo`, since `duo` has not been shifted up yet to allow storage of it.
                div >>= 1;
                shift -= 1;
                div_neg_add_1 = div.wrapping_neg().wrapping_add(1);
                div_sub_1 = div.wrapping_sub(1);
                let sub = duo.wrapping_sub(div);
                let tmp = 1 << shift;
                mask = tmp - 1;
                if (sub as $iX) >= 0 {
                    duo = sub;
                    quo |= tmp;
                }
            } else {
                mask = quo - 1;
                div_neg_add_1 = div.wrapping_neg().wrapping_add(1);
                div_sub_1 = div.wrapping_sub(1);
            }
            println!("duo:{:08b}, div:{:08b}", duo, div);
            println!("div_neg_add_1:{:08b}, div_sub_1:{:08b}, shift: {}, quo: {:08b}", div_neg_add_1, div_sub_1, shift, quo);
            for i in 0..shift {
                println!("duo:{:08b}", duo);
                let mut s = "  ".to_owned();
                for _ in 0..($n - i) {
                    s.push(' ');
                }
                s.push('^');
                println!("{}", s);
                if (duo as $iX) < 0 {
                    // Negated binary long division step.
                    duo = div_sub_1.wrapping_add(duo.wrapping_shl(1));
                } else {
                    // Regular long division step.
                    duo = div_neg_add_1.wrapping_add(duo.wrapping_shl(1));
                }
            }
            println!("duo:{:08b}, div:{:08b}", duo, div);
            if (duo as $iX) < 0 {
                duo = div_sub_1.wrapping_add(duo);
            }
            println!("duo:{:08b}, div:{:08b}", duo, div);
            println!("mask:{:08b}", mask);
            println!("quo:{:08b}", (duo & mask) | quo);
            println!("rem:{:08b}", duo >> shift);

            ((duo & mask) | quo, duo >> shift)
        }

        /// Computes the quotient and remainder of `duo` divided by `div` and returns them as a
        /// tuple.
        ///
        ///
        /// # Panics
        ///
        /// When attempting to divide by zero, this function will panic.
        $(
            #[$signed_attr]
        )*
        pub fn $signed_name(duo: $iX, div: $iX) -> ($iX, $iX) {
            match (duo < 0, div < 0) {
                (false, false) => {
                    let t = $unsigned_name(duo as $uX, div as $uX);
                    (t.0 as $iX, t.1 as $iX)
                },
                (true, false) => {
                    let t = $unsigned_name(duo.wrapping_neg() as $uX, div as $uX);
                    ((t.0 as $iX).wrapping_neg(), (t.1 as $iX).wrapping_neg())
                },
                (false, true) => {
                    let t = $unsigned_name(duo as $uX, div.wrapping_neg() as $uX);
                    ((t.0 as $iX).wrapping_neg(), t.1 as $iX)
                },
                (true, true) => {
                    let t = $unsigned_name(duo.wrapping_neg() as $uX, div.wrapping_neg() as $uX);
                    (t.0 as $iX, (t.1 as $iX).wrapping_neg())
                },
            }
        }
    }
}
