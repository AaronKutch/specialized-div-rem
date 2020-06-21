macro_rules! impl_binary_long {
    (
        $unsigned_name:ident, // name of the unsigned division function
        $signed_name:ident, // name of the signed division function
        $normalization_shift:ident, // function for finding the normalization shift
        $n:tt, // the number of bits in a $iX or $uX
        $uX:ident, // unsigned integer type for the inputs and outputs of `$unsigned_name`
        $iX:ident, // signed integer type for the inputs and outputs of `$signed_name`
        $($unsigned_attr:meta),*; // attributes for the unsigned function
        $($signed_attr:meta),* // attributes for the signed function
    ) => {
        /// Computes the quotient and remainder of `duo` divided by `div` and returns them as a
        /// tuple.
        ///
        /// This uses binary long division with no calls to smaller divisions, and is designed for
        /// CPUs without fast division hardware. The algorithm used is designed for architectures
        /// without predicated instructions. If more performance is wanted for a predicated
        /// architecture, a custom assembly routine using one of the algorithms described in the
        /// documentation of this function should be used instead.
        ///
        /// # Panics
        ///
        /// When attempting to divide by zero, this function will panic.
        $(
            #[$unsigned_attr]
        )*
        pub fn $unsigned_name(duo: $uX, div: $uX) -> ($uX, $uX) {
            // handle edge cases before finding the normalization shift
            if div == 0 {
                panic!("attempt to divide by zero")
            }
            if duo < div {
                return (0, duo)
            }
            if (duo - div) < div {
                return (1, duo - div)
            }
            let mut duo = duo;
            let mut shift: usize = $normalization_shift(duo, div);

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
                //println!("duo:{:08b}, div:{:08b}, sub:{:08b}, quo:{:08b}, shift:{}", duo, div, sub, quo, shift);
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
            // division algorithm that does not rely on carry flags, add-with-carry, or SWAR
            // complications.
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

            // This is the SWAR (SIMD within in a register) binary long division algorithm.
            // This combines several ideas of the above algorithms:
            //  - If `duo` is shifted left instead of shifting `div` right like in the 3 instruction
            //    restoring division algorithm, some architectures can do the shifting and
            //    subtraction step in one instruction.
            //  - `quo` can be constructed by adding powers-of-two to it or shifting it left by one
            //    and adding one.
            //  - Every time `duo` is shifted left, there is another unused 0 bit shifted into the
            //    LSB, so what if we use those bits to store `quo`?
            // Through a complex setup, it is possible to manage `duo` and `quo` in the same
            // register, and perform one step with 2 or 3 instructions. The only major downsides are
            // that there is significant setup, `duo < div_original` checks are impractical once
            // SWAR is initiated, and the number of division steps taken has to be exact.
            /*
            // first step. The quotient bit is stored in `quo` for now
            let div_original = div;
            let mut div: $uX = (div << shift);
            duo = duo.wrapping_sub(div);
            let mut quo: $uX = 1 << shift;
            if duo < div_original {
                return (quo, duo);
            }

            let mask: $uX;
            if div >= (1 << ($n - 1)) {
                // deal with same edge case as the 3 instruction restoring division algorithm, but
                // the quotient bit from this step also has to be stored in `quo`
                div >>= 1;
                shift -= 1;
                let tmp = 1 << shift;
                mask = tmp - 1;
                let sub = duo.wrapping_sub(div);
                if (sub as $iX) >= 0 {
                    // restore
                    duo = sub;
                    quo |= tmp;
                }
                if duo < div_original {
                    return (quo, duo);
                }
            } else {
                mask = quo - 1;
            }
            // There is now room for quotient bits in `duo`.

            // When subtracted from `duo.wrapping_shl(1)`, this adds a quotient bit to the least
            // significant bit. When added to `duo`, it subtracts away the quotient bit.
            let div: $uX = div.wrapping_sub(1);
            // comment out the for loop and uncomment the other lines of code to enable unrolling
            //let mut i = shift as isize;
            //unroll!($n, i, {
            for _ in 0..shift {
                duo = duo.wrapping_shl(1).wrapping_sub(div);
                if (duo as $iX) < 0 {
                    duo = duo.wrapping_add(div);
                }
            }
            //});
            return ((duo & mask) | quo, duo >> shift);
            */

            // The problem with the conditional restoring SWAR algorithm above is that it requires
            // assembly code to bring out its full unrolled potential. On architectures without
            // predicated instructions, the code gen is especially bad. We need a default software
            // division algorithm that is guaranteed to get good code gen for the central loop.

            // For non-SWAR algorithms, there is a way to do binary long division without
            // predication or even branching. This involves creating a mask from the sign bit and
            // performing different kinds of steps using that.
            //
            // (example of a step for a branchless restoring algorithm with no need for predication)
            // let sub = duo.wrapping_sub(div);
            // let sign_mask = !(((sub as $iX).wrapping_shr($n - 1)) as $uX);
            // duo -= div & sign_mask;
            // quo |= sign_mask & pow;
            //
            // However, it requires about 4 extra operations (smearing the sign bit, negating the
            // mask, and applying the mask twice) on top of the operations done by the actual
            // algorithm. With SWAR however, just 2 extra operations are needed, making it
            // practical and even the most optimal algorithm for some architectures.
            //
            // duo = duo.wrapping_shl(1).wrapping_sub(div);
            // let mask = (duo as $iX).wrapping_shr($n - 1) as $uX;
            // duo = duo.wrapping_add(div & mask);
            //
            // What we do is use custom assembly for predicated architectures that need software
            // division, and for the default algorithm use a mask based restoring SWAR algorithm
            // without conditionals or branches. On almost all architectures, this Rust code is
            // guaranteed to compile down to 5 assembly instructions or less for each step, along
            // with a small amount of branching overhead to get the exact number of steps required.
            // The `unroll!` macro has similar performance to Duff's device, without requiring
            // compilers to be smart enough to use variable assembly jumps.

            // standard opening for SWAR algorithm with first step and edge case handling
            let div_original = div;
            let mut div: $uX = (div << shift);
            duo = duo.wrapping_sub(div);
            let mut quo: $uX = 1 << shift;
            if duo < div_original {
                return (quo, duo);
            }
            let mask: $uX;
            if div >= (1 << ($n - 1)) {
                div >>= 1;
                shift -= 1;
                let tmp = 1 << shift;
                mask = tmp - 1;
                let sub = duo.wrapping_sub(div);
                if (sub as $iX) >= 0 {
                    duo = sub;
                    quo |= tmp;
                }
                if duo < div_original {
                    return (quo, duo);
                }
            } else {
                mask = quo - 1;
            }

            div = div.wrapping_sub(1);
            // central loop with unrolling
            let mut i = shift as isize;
            unroll!($n, i, {
            // for _ in 0..shift {
                duo = duo.wrapping_shl(1).wrapping_sub(div);
                let mask = (duo as $iX).wrapping_shr($n - 1) as $uX;
                duo = duo.wrapping_add(div & mask);
            // }
            });
            return ((duo & mask) | quo, duo >> shift);

            // miscellanious binary long division algorithms that might be better for specific
            // architectures

            // Another kind of long division uses an interesting fact that `div` and `pow` can be
            // negated when `duo` is negative to perform a "negated" division step that works in
            // place of any normalization mechanism. This is a non-restoring division algorithm that
            // is very similar to the non-restoring division algorithms that can be found on the
            // internet, except there is only one test for `duo < 0`. The subtraction from `quo` can
            // be viewed as shifting the least significant set bit right (e.x. if we enter a series
            // of negated binary long division steps starting with `quo == 0b1011_0000` and
            // `pow == 0b0000_1000`, `quo` will progress like this: 0b1010_1000, 0b1010_0100,
            // 0b1010_0010, 0b1010_0001).
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
                    // Normal long division step.
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

            // The most promising division algorithm combines the nonrestoring algorithm above with
            // SWAR techniques that makes the only difference between steps be negation of `div`.
            // If there was an architecture with an instruction that negated inputs to an adder
            // based on conditionals, and in place shifting (or a three input addition operation
            // that can have `duo` as two of the inputs to effectively shift it left by 1), then a
            // single instruction central loop is possible. Microarchitectures often have inputs to
            // their ALU that can invert the arguments and carry in of adders, but the architectures
            // unfortunately do not have an instruction to dynamically invert this input based on
            // conditionals.
            /*
            let div_original = div;
            let mut div: $uX = (div << shift);
            duo = duo.wrapping_sub(div);
            let mut quo: $uX = 1 << shift;
            if duo < div_original {
                return (quo, duo);
            }
            let mask: $uX;
            if div >= (1 << ($n - 1)) {
                div >>= 1;
                shift -= 1;
                let tmp = 1 << shift;
                let sub = duo.wrapping_sub(div);
                if (sub as $iX) >= 0 {
                    // restore
                    duo = sub;
                    quo |= tmp;
                }
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
                // note: the `wrapping_shl(1)` can be factored out, but would require another
                // restoring division step to prevent `(duo as $iX)` from overflowing
                if (duo as $iX) < 0 {
                    // Negated binary long division step.
                    duo = duo.wrapping_shl(1).wrapping_add(div);
                } else {
                    // Normal long division step.
                    duo = duo.wrapping_shl(1).wrapping_sub(div);
                }
                i += 1;
                if i == shift {
                    break;
                }
            }
            if (duo as $iX) < 0 {
                // Restore. This was not needed in the original nonrestoring algorithm because of
                // the `duo < div_original` checks.
                duo = duo.wrapping_add(div);
            }
            return ((duo & mask) | quo, duo >> shift);
            */
        }

        /// Computes the quotient and remainder of `duo` divided by `div` and returns them as a
        /// tuple.
        ///
        /// This uses binary long division with no calls to smaller divisions, and is designed for
        /// CPUs without fast division hardware. The algorithm used is designed for architectures
        /// without predicated instructions. If more performance is wanted for a predicated
        /// architecture, a custom assembly routine using one of the algorithms described in the
        /// documentation of this function should be used instead.
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
