macro_rules! impl_binary_long {
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
        /// This uses binary long division with no calls to smaller divisions, and is designed for
        /// CPUs without fast division hardware.
        ///
        /// # Panics
        ///
        /// When attempting to divide by zero, this function will panic.
        $(
            #[$unsigned_attr]
        )*
        pub fn $unsigned_name(duo: $uX, div: $uX) -> ($uX, $uX) {
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

            // We have to find the leading zeros of `div` to know where its most significant bit
            // it to even begin binary long division. It is also good to know where the most
            // significant bit of `duo` is so that useful work can be started instead of shifting
            // `div` for all possible quotients (many division steps are wasted if
            // `duo.leading_zeros()` is large and `div` starts out being shifted all the way to the
            // most significant bit). Aligning the most significant bit of `div` and `duo` could be
            // done by shifting `div` left by `div.leading_zeros() - duo.leading_zeros()`, but many
            // CPUs without division hardware also do not have single instructions for calculating
            // `leading_zeros`. Instead of software doing two bisections to find the two
            // `leading_zeros`, we do one bisection to find
            // `div.leading_zeros() - duo.leading_zeros()` without actually knowing either of the
            // leading zeros values.

            // If we shift `duo` right and subtract `div` from the shifted value and the result is
            // negative, then the most significant bit of `duo` is even with or has passed the most
            // significant bit of `div` and the shift can be decreased. Otherwise, the most
            // significant bit of `duo` has not passed that of `div` and the shift can be increased.
            //
            // Example: finding the aligning shift for dividing 178u8 (0b10110010) by 6u8 (0b110)
            // first loop:
            // level: 2, shift: 4
            // duo >> shift: 0b00001011
            //          div: 0b00000110
            //             - __________
            //          sub: 0b00000101
            // sub is positive, so increase the shift amount by the current level of bisection.
            // second loop:
            // level: 1, shift: 6
            // duo >> shift: 0b00000010
            //          div: 0b00000110
            //             - __________
            //          sub: 0b11111100
            // sub is negative, so decrease the shift.
            //
            // The tricky part is when the significant bits are even with each other. In that case,
            // `duo.wrapping_sub(div)` could be positive or negative and this algorithm falls into a
            // repeating cycle between two values of `shift` (in this case, it will cycle between
            // shifts of 4 and 5). The smaller of the two shift values turns out to always be valid
            // for starting long division.
            //
            // (the last step uses `level = 1` again)
            // level: 1, shift: 5
            // duo >> shift: 0b00000101
            //          div: 0b00000110
            //             - __________
            //          sub: 0b11111111
            // sub is negative, so decrease the shift, otherwise keep the shift the same.
            //
            let mut level = $n / 4;
            let mut shift = $n / 2;
            let mut duo = duo;
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
                    // if `(sub as $iX) < 0`, it involves cases where sub is smaller than `div`
                    // when the most significant bits are aligned, like in the example above.
                    // Then, we can immediately do a long division step without needing a
                    // normalization check. There is an edge case we avoid by using a `sub < 0`
                    // comparison rather than a `sub <= 0` comparison on the branch:
                    // if duo = 0b1001 and div = 0b0100, we could arrive to this branch with
                    // shift: 1
                    // duo >> shift: 0b00000100
                    //          div: 0b00000100
                    //             - __________
                    //          sub: 0b00000000
                    // the problem with this is that `duo >> shift` is shifting off a set bit
                    // that makes `duo >= 2*(div << shift)`, which would break binary division steps
                    // assuming normalization, but this cannot happen with `sub < 0`.
                    //
                    // If `(sub as $iX) >= 0`, it involves cases where `sub >= div` when the most
                    // significant bits are aligned. We know that the current shift is the smaller
                    // shift in the cycle and can automatically be part of a long division step.
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
            
            // There are many variations of binary division algorithm that could be used. This
            // documentation gives a tour of different methods so that future readers wanting to
            // optimize further do not have to painstakingly derive them. The SWAR variation is
            // especially hard to understand without reading the less convoluted methods first.

            // You may notice that
            // a `duo < div_original` check is included in all these algorithms. A critical
            // optimization that many algorithms I see online miss is handling of quotients that
            // will turn out to have many trailing zeros or many leading zeros. This happens in
            // cases of exact or close-to-exact divisions, divisions by power of two, and in cases
            // where the quotient is small. The `duo < div_original` check handles these cases of
            // early returns and ends up replacing other kinds of mundane checks that normally
            // terminate a binary division algorithm.
            //
            // Something you may see in other algorithms that is not special-cased here is checks
            // for division by powers of two. The `duo < div_original` check handles this case and
            // more, however it can be checked up front before the bisection using the
            // `((div > 0) && ((div & (div - 1)) == 0))` trick. This is not special-cased because
            // compilers should handle most cases where divisions by power of two occur, and we do
            // not want to add on a few cycles for every division operation just to save a few
            // cycles rarely.

            // The following example is the most straightforward translation from the way binary
            // long division is typically visualized:
            // dividing 178u8 (0b10110010) by 6u8 (0b110), continued from the end of the bisection
            // algorithm where we found `shift`. `div` is shifted left 4 during initialization.
            // duo = 0b10110010
            // div = 0b01100000
            //     - __________
            // sub = 0b01010010
            // quo = 0b00000001 (set because `sub >= 0`)
            //
            // duo = 0b01010010 (also set to `sub` because sub >= 0)
            // div = 0b00110000 (shifted right one)
            //     - __________
            // sub = 0b00100010
            // quo = 0b00000011
            //
            // duo = 0b00100010
            // div = 0b00011000
            //     - __________
            // sub = 0b00001010
            // quo = 0b00000111
            //
            // duo = 0b00001010
            // div = 0b00001100
            //     - __________
            // sub = 0b11111110 (now `sub < 0`, which means we are not normalized and need to skip
            // quo = 0b00001110  updating `duo` or setting a bit in `quo`)
            //
            // duo = 0b00001010
            // div = 0b00000110
            //     - __________
            // sub = 0b00000100
            // quo = 0b00011101
            //
            // duo = 0b00000100
            // now `shift == 0` so `quo` is now the correct quotient of 29u8 and `duo` is the
            // remainder 4u8
            /*
            let div_original = div;
            let mut div = div << shift;
            let mut quo = 0;
            loop {
                let sub = duo.wrapping_sub(div);
                // it is recommended to use `println!`s like this if functionality is unclear
                //println!("duo:{:08b}, div:{:08b}, sub:{:08b}, shift:{}", duo, div, sub, shift);
                if (sub as $iX) >= 0 {
                    duo = sub;
                    quo |= 1;
                    if duo < div_original {
                        // this branch is optional for this particular algorithm
                        return (quo << shift, duo)
                    }
                }
                if shift == 0 {
                    return (quo, duo)
                }
                shift -= 1;
                div >>= 1;
                quo <<= 1;
            }
            */

            // In the above algorithm, notice that `quo` is gradually being shifted up and having a
            // 1 OR'ed into its least significant bit. One some architectures, this can be
            // accomplished in one add-with-carry instruction that adds `quo` to itself to shift up
            // by one, and the least significant bit is set if the carry flag is clear. However,
            // there are many architectures without such functionality (e.g. RISC-V without the `M`
            // extension). Instead of shifting `quo`, we could shift a power-of-two `pow` right on
            // each step, and bitwise-or it in place with `quo` when normalized. The
            // `duo < div_original` check can be used in place of the `shift == 0` check, and `pow`
            // can be shifted right instead of needing to recalculate it from `1 << shift`, so
            // `shift` can be removed entirely. The following algorithm is the fastest restoring
            // division algorithm that does not rely on carry flags or add-with-carry.

            /*
            // Perform one binary long division step on the already normalized arguments, and setup
            // all the variables.
            let div_original = div;
            let mut div: $uX = (div << shift);
            let mut pow: $uX = 1 << shift;
            let mut quo: $uX = pow;
            duo = duo.wrapping_sub(div);
            if duo < div_original {
                return (quo, duo);
            }
            div >>= 1;
            pow >>= 1;
            loop {
                let sub = duo.wrapping_sub(div);
                if (sub as $iX) >= 0 {
                    duo = sub;
                    quo |= pow;
                    if duo < div_original {
                        return (quo, duo)
                    }
                }
                pow >>= 1;
                div >>= 1;
            }
            */

            // There is a way to do restoring binary long division without branching or predication,
            // but it requires about 4 extra operations (smearing the sign bit, negating the mask,
            // and applying the mask twice):
            //
            // (example of central loop)
            // let sub = duo.wrapping_sub(div);
            // let sign_mask = !(((sub as $iX) >> ($n - 1)) as $uX);
            // duo -= div & sign_mask;
            // quo |= sign_mask & pow;
            //
            // This might be better on CPUs with incredibly long pipelines, but the smaller CPUs
            // that need software division probably do not have the 8 cycle branch miss penalties
            // needed to break even (assuming that a branch miss occurs on half of steps). Branches
            // would still need to be inserted at regular intervals to make sure exact divisions are
            // fast.

            // If the architecture has flags and predicated arithmetic instructions, it is possible
            // to do binary long division without branching and in only 3 or 4 instructions. This is
            // a variation of a 3 instruction central loop from
            // http://www.chiark.greenend.org.uk/~theom/riscos/docs/ultimate/a252div.txt.
            //
            // What allows doing division in only 3 instructions is realizing that instead of
            // keeping `duo` in place and shifting `div` right to align bits, `div` can be kept in
            // place and `duo` can be shifted left. This would not normally save any instructions
            // and just cause more edge case problems and make `duo < div_original` tests harder.
            // However, some architectures have an option to shift an argument in an arithmetic
            // operation, meaning `duo` can be shifted left and subtracted from in one instruction.
            // `div` never has to be written to in the loop. The other two instructions are updating
            // `quo` and undoing the subtraction if it turns out things were not normalized.
            /*
            // Perform one binary long division step on the already normalized arguments, and setup
            // all the variables.
            let div_original = div;
            let mut div: $uX = (div << shift);
            let mut quo: $uX = 1;
            duo = duo.wrapping_sub(div);
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
            return (quo, duo >> shift);
            */

            // Another kind of long division uses an interesting fact that `div` and `pow` can be
            // negated when `duo` is negative to perform a "negated" division step that works in
            // place of any normalization mechanism. This is a non-restoring division algorithm that
            // is very similar to the non-restoring division algorithms that can be found on the
            // internet, except there is only one test for `duo < 0`.
            /*
            let div_original = div;
            let mut div: $uX = (div << shift);
            let mut pow: $uX = 1 << shift;
            let mut quo: $uX = pow;

            duo = duo.wrapping_sub(div);
            if duo < div_original {
                return (quo, duo);
            }
            div >>= 1;
            pow >>= 1;
            loop {
                if (duo as $iX) < 0 {
                    // Negated binary long division step.
                    duo = duo.wrapping_add(div);
                    quo = quo.wrapping_sub(pow);
                    pow >>= 1;
                    div >>= 1;
                } else {
                    // Regular long division step.
                    if duo < div_original {
                        return (quo, duo)
                    }
                    duo = duo.wrapping_sub(div);
                    quo = quo.wrapping_add(pow);
                    pow >>= 1;
                    div >>= 1;
                }
            }
            */

            // This final variation is the SWAR (SIMD within in a register) binary long division
            // algorithm. This combines several ideas of the above algorithms:
            //  - If `duo` is shifted left instead of shifting `div` right like in the 3 instruction
            //    restoring division algorithm, some architectures can do the shifting and
            //    subtraction step in one instruction.
            //  - `quo` can be constructed by adding powers-of-two to it, shifting
            //    it left by one and adding one, or shifting left and subtracting one in a
            //    non-restoring step.
            //  - The non-restoring division algorithm is very symmetric when adding and subtracting
            //    to `duo` in quo`.
            //  - Every time `duo` is shifted left, there is another unused 0 bit shifted into the
            //    LSB, so what if we use those bits to store `quo`?
            // Through a complex setup, it is possible to manage `duo` and `quo` in the same
            // register, and perform one step with one instruction. Even better, add-with-carry is
            // not needed. The only major downsides are that `duo < div_original` checks are
            // impractical, and the number of division steps taken has to be exact.

            let div_original = div;
            let mut div: $uX = (div << shift);
            duo = duo.wrapping_sub(div);
            // Until `duo` has been shifted left, there is no room for quotient bits in `duo`, so
            // we store the bits here and add them on at the end.
            let mut quo: $uX = 1 << shift;
            if duo < div_original {
                return (quo, duo);
            }

            let mask: $uX;
            if div >= (1 << ($n - 1)) {
                // deal with same edge case as the 3 instruction restoring division algorithm, but
                // the quotient bit from this step has to be stored in `quo`
                div >>= 1;
                shift -= 1;
                let tmp = 1 << shift;
                let sub = duo.wrapping_sub(div);
                if (sub as $iX) >= 0 {
                    // restore
                    duo = sub;
                    quo |= tmp;
                }
                // need this so that a check at the beginning of loops is not needed
                if duo < div_original {
                    return (quo, duo);
                }
                mask = tmp - 1;
            } else {
                mask = quo - 1;
            }

            // the subtracted one acts as a negative one for the quotient inside `duo`
            let div: $uX = div.wrapping_sub(1);
            let mut i = 0;
            loop {
                // note: the `duo.wrapping_shl(1)` cannot be factored out because it would require
                // another restoring division step to prevent `(duo as $iX)` from overflowing
                if (duo as $iX) < 0 {
                    // Negated binary long division step.
                    duo = duo.wrapping_shl(1).wrapping_add(div);
                } else {
                    // Regular long division step.
                    duo = duo.wrapping_shl(1).wrapping_sub(div);
                }
                i += 1;
                if i == shift {
                    break;
                }
            }
            if (duo as $iX) < 0 {
                // Extra negated binary long division step. This is not needed in the original
                // nonrestoring algorithm because of the `duo < div_original` checks
                duo = duo.wrapping_add(div);
            }

            return ((duo & mask) | quo, duo >> shift);
        }

        /// Computes the quotient and remainder of `duo` divided by `div` and returns them as a
        /// tuple.
        ///
        /// This uses binary long division with no calls to smaller divisions, and is designed for
        /// CPUs without fast division hardware.
        ///
        /// # Panics
        ///
        /// When attempting to divide by zero, this function will panic.
        $(
            #[$signed_attr]
        )*
        pub fn $signed_name(duo: $iX, div: $iX) -> ($iX, $iX) {
            // There is a way of doing this without any branches, but the branching method below
            // requires several fewer operations, and compilers can easily inline particular
            // branches if it knows some inputs will never be negative.
            /*
            let duo_s = duo >> ($n - 1);
            let div_s = div >> ($n - 1);
            let duo = (duo ^ duo_s).wrapping_sub(duo_s);
            let div = (div ^ div_s).wrapping_sub(div_s);
            let quo_s = duo_s ^ div_s;
            let rem_s = duo_s;
            let tmp = $unsigned_name(duo as $uX, div as $uX);
            (
                ((tmp.0 as $iX) ^ quo_s).wrapping_sub(quo_s),
                ((tmp.1 as $iX) ^ rem_s).wrapping_sub(rem_s),
            )
            */

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
