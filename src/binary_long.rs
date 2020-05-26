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
        /// CPUs without fast division hardware for small integers. This algorithm is designed for
        /// architectures that do not have predicated arithmetic instructions or flags, such as
        /// RISC-V (without the M extension). Use `_carry_left` instead for architectures with the
        /// necessary arithmetic instructions.
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
            // This eliminates cases where the most significant bits of `duo` or `div` are set.
            // Signed comparisons can then be used to determine overflow without needing carry
            // flags.
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
                    // that makes `duo >= 2*(div << shift)`, which would break the binary
                    // division step below, but this cannot happen with `sub < 0`.
                    //
                    // Otherwise, it involves cases where `sub >= div` when the most significant
                    // bits are aligned. We know that the current shift is the smaller shift in
                    // the cycle and can automatically be part of a long division step.
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
            // documentation explains why the final variation is what is chosen. You may notice that
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
            // now `shift == 0` so `quo` is now the correct quotient of 29 and `duo` is the
            // remainder
            /*
            let mut div = div << shift;
            let mut quo = 0;
            loop {
                let sub = duo.wrapping_sub(div);
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
            // by one, and the least significant bit is set if the carry flag is clear. However, we
            // are targeting architectures without such functionality. Instead of shifting `quo`, we
            // shift a power-of-two `pow` right on each step, and OR it in place with `quo` when
            // normalized. The `duo < div_original` check can be used in place of the `shift == 0`
            // check, so `shift` can be removed entirely. It results in 5 instructions being
            // executed for unnormalized steps, and 8 instructions for normalized steps. Unrolled,
            // `quo |= pow` can be done with a constant, and the unconditional branch can be
            // eliminated, eliminating two instructions. Assuming both kinds of steps are equally
            // likely, each step takes 4 assembly instructions on average.

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

            // Another kind of long division uses an interesting fact that `div` and `pow` can be
            // negated when `duo` is negative to perform a "negated" division step that works in
            // place of any normalization mechanism. Unfortunately, it requires about the same
            // number of instructions for one step as the other division algorithm. This is kept
            // here in case it is useful for some case.
            /*
            // note that the two most significant bits of `duo` must be cleared and the arguments
            // normalized before reaching this point.
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
                if duo < 0 {
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

            // Finally, there is a way to do binary long division without branching or predication,
            // but it requires about 4 extra operations (smearing the sign bit, negating the mask,
            // applying the mask twice):
            //
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
        }

        /// Computes the quotient and remainder of `duo` divided by `div` and returns them as a
        /// tuple.
        ///
        /// This uses binary long division with no calls to smaller divisions, and is designed for
        /// CPUs without fast division hardware for small integers. This algorithm is designed for
        /// architectures that do not have predicated arithmetic instructions or flags, such as
        /// RISC-V (without the M extension). Use `_carry_left` instead for architectures with the
        /// necessary arithmetic instructions.
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
