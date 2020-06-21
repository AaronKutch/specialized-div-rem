//! miscellanious functions and macros used in the rest of the crate

#[cfg(test)]
use rand::random;

/// Returns the number of leading binary zeros in `x`.
///
/// Counting leading zeros is performance critical to certain division algorithms and more. This is
/// a fast software routine for CPU architectures without an assembly instruction to quickly
/// calculate `leading_zeros`.
pub const fn leading_zeros(x: usize) -> usize {
    // Note: This routine produces the correct value for `x == 0`. Zero is probably common enough
    // that it could warrant adding a zero check at the beginning, but it was decided not to do
    // this. This code is meant to be the routine for `compiler-builtins` functions like `__clzsi2`
    // which have a precondition that `x != 0`. Compilers will insert the check for zero in cases
    // where it is needed.

    // The base idea is to mask the higher bits of `x` and compare them to zero to bisect the number
    // of leading zeros. For an example, if we are finding the leading zeros of a `u8`, we could
    // check if `x & 0b1111_0000` is zero. If it is zero, then the number of leading zeros is at
    // least 4, otherwise it is less than 4. If `(x & 0b1111_0000) == 0`, then we could branch to
    // another check for if `x & 0b1111_1100` is zero. If `(x & 0b1111_1100) != 0`, then the number
    // of leading zeros is at least 4 but less than 6. One more bisection with `x & 0b1111_1000`
    // determines the number of leading zeros to be 4 if `(x & 0b1111_1000) != 0` and 5 otherwise.
    //
    // However, we do not want to have 6 levels of bisection to 64 leaf nodes and hundreds of bytes
    // of instruction code if `usize` has a bit width of 64. It is possible for all branches of the
    // bisection to use the same code path by conditional shifting and focusing on smaller parts:
    /*
    let mut x = x;
    // temporary
    let mut t: usize;
    // The number of potential leading zeros, assuming that `usize` has a bitwidth of 64 bits
    let mut z: usize = 64;

    t = x >> 32;
    if t != 0 {
        // one of the upper 32 bits is set, so the 32 lower bits are now irrelevant and can be
        // removed from the number of potential leading zeros
        z -= 32;
        // shift `x` so that the next step can deal with the upper 32 bits, otherwise the lower 32
        // bits would be checked by the next step.
        x = t;
    }
    t = x >> 16;
    if t != 0 {
        z -= 16;
        x = t;
    }
    t = x >> 8;
    if t != 0 {
        z -= 8;
        x = t;
    }
    t = x >> 4;
    if t != 0 {
        z -= 4;
        x = t;
    }
    t = x >> 2;
    if t != 0 {
        z -= 2;
        x = t;
    }
    // combine the last two steps
    t = x >> 1;
    if t != 0 {
        return z - 2;
    } else {
        return z - x;
    }
    */
    // The above method has short branches in it which can cause pipeline problems on some
    // platforms, or the compiler removes the branches but at the cost of huge instruction counts
    // from heavy bit manipulation. We can remove the branches by turning `(x & constant) != 0`
    // boolean expressions into an integer.

    // Adapted from LLVM's `compiler-rt/lib/builtins/clzsi2.c`. The original sets the number of
    // zeros `z` to be 0 and add to that:
    //
    // // If the upper bits are zero, set `t` to `1 << level`
    // let t = (((x & const) == 0) as usize) << level;
    // // If the upper bits are zero, the right hand side expression cancels to zero and no shifting
    // // occurs
    // x >>= (1 << level) - t;
    // // If the upper bits are zero, `1 << level` is added to `z`
    // z += t;
    //
    // It saves some instructions to start `z` at 64 and subtract from that with a negated process
    // instead:
    //
    // // use `!=` comparison instead
    // let t = (((x & const) != 0) as usize) << level;
    // // shift if the upper bits are not zero
    // x >>= t;
    // // subtract from the number of potential leading zeros
    // z -= t;

    let mut x = x;
    // The number of potential leading zeros
    let mut z = {
        #[cfg(target_pointer_width = "64")]
        {
            64
        }
        #[cfg(target_pointer_width = "32")]
        {
            32
        }
        #[cfg(target_pointer_width = "16")]
        {
            16
        }
    };

    // a temporary
    let mut t: usize;

    #[cfg(target_pointer_width = "64")]
    {
        t = (((x & 0xFFFF_FFFF_0000_0000) != 0) as usize) << 5;
        x >>= t;
        z -= t;
    }

    #[cfg(any(target_pointer_width = "32", target_pointer_width = "64"))]
    {
        t = (((x & 0xFFFF_0000) != 0) as usize) << 4;
        x >>= t;
        z -= t;
    }

    t = (((x & 0xFF00) != 0) as usize) << 3;
    x >>= t;
    z -= t;

    t = (((x & 0xF0) != 0) as usize) << 2;
    x >>= t;
    z -= t;

    t = (((x & 0b1100) != 0) as usize) << 1;
    x >>= t;
    z -= t;

    t = ((x & 0b10) != 0) as usize;
    x >>= t;
    z -= t;

    // All bits except LSB are guaranteed to be zero by this point. If `x != 0` then `x == 1` and
    // subtracts a potential zero from `z`.
    z - x
}

#[test]
fn leading_zeros_test() {
    // binary fuzzer
    let mut x = 0usize;
    let mut ones: usize;
    // creates a mask for indexing the bits of the type
    let bit_indexing_mask = usize::MAX.count_ones() - 1;
    for _ in 0..1000 {
        for _ in 0..4 {
            let r0: u32 = bit_indexing_mask & random::<u32>();
            ones = !0 >> r0;
            let r1: u32 = bit_indexing_mask & random::<u32>();
            let mask = ones.rotate_left(r1);
            match (random(), random()) {
                (false, false) => x |= mask,
                (false, true) => x &= mask,
                (true, _) => x ^= mask,
            }
        }
        if leading_zeros(x) != (x.leading_zeros() as usize) {
            panic!(
                "x: {}, expected: {}, found: {}",
                x,
                x.leading_zeros(),
                leading_zeros(x)
            );
        }
    }
}

/// Creates multiple intensive test functions for division functions of a certain size
macro_rules! test {
    (
        $n:expr, // the number of bits in a $iX or $uX
        $uX:ident, // unsigned integer that will be shifted
        $iX:ident, // signed version of $uX
        // list of triples of the test name, the unsigned division function, and the signed
        // division function
        $($test_name:ident, $unsigned_name:ident, $signed_name:ident);+
    ) => {
        $(
            #[test]
            fn $test_name() {
                // Brute force fuzzer that checks all possible single continuous strings of ones
                // (e.x. 0b00111000, 0b11110000, 0b01111110). This test is critical for finding
                // corner cases that the randomized fuzzer may miss.

                // This is reversed so that small values appear first, which helps development
                for lhs_len in (0..$n).rev() {
                    for lhs_shift in 0..=lhs_len {
                        for rhs_len in (0..$n).rev() {
                            for rhs_shift in 0..=rhs_len {
                                let lhs = (!0 >> lhs_len) << lhs_shift;
                                let rhs = (!0 >> rhs_len) << rhs_shift;

                                if rhs != 0 {
                                    let (quo, rem) = $unsigned_name(lhs, rhs);
                                    if lhs != rhs.wrapping_mul(quo).wrapping_add(rem) {
                                        panic!(
                                            "unsigned division function failed with lhs:{} rhs:{} \
                                            expected:({}, {}) found:({}, {})",
                                            lhs,
                                            rhs,
                                            lhs.wrapping_div(rhs),
                                            lhs.wrapping_rem(rhs),
                                            $unsigned_name(lhs, rhs).0,
                                            $unsigned_name(lhs, rhs).1
                                        );
                                    }

                                    // test the signed division function also
                                    let lhs = lhs as $iX;
                                    let rhs = rhs as $iX;
                                    let (quo, rem) = $signed_name(lhs, rhs);
                                    if lhs != rhs.wrapping_mul(quo).wrapping_add(rem) {
                                        panic!(
                                            "signed division function failed with lhs:{} rhs:{} \
                                            expected:({}, {}) found:({}, {})",
                                            lhs,
                                            rhs,
                                            lhs.wrapping_div(rhs),
                                            lhs.wrapping_rem(rhs),
                                            $signed_name(lhs, rhs).0,
                                            $signed_name(lhs, rhs).1
                                        );
                                    }
                                }
                            }
                        }
                    }
                }

                // Specially designed random fuzzer
                let mut lhs: $uX = 0;
                let mut rhs: $uX = 0;
                // all ones constant
                let ones: $uX = !0;
                // Alternating ones and zeros (e.x. 0b1010101010101010). This catches second-order
                // problems that might occur for algorithms with two modes of operation (potentially
                // there is some invariant that can be broken for large `duo` and maintained via
                // alternating between modes, breaking the algorithm when it reaches the end).
                let mut alt_ones: $uX = 1;
                for _ in 0..($n / 2) {
                    alt_ones <<= 2;
                    alt_ones |= 1;
                }
                // creates a mask for indexing the bits of the type
                let bit_indexing_mask = $n - 1;
                for _ in 0..1_000_000 {
                    // randomly OR, AND, and XOR randomly sized and shifted continuous strings of
                    // ones with `lhs` and `rhs`. XOR is performed most often because OR and AND
                    // tend to be destructive.
                    let r0: u32 = bit_indexing_mask & random::<u32>();
                    let r1: u32 = bit_indexing_mask & random::<u32>();
                    let mask = ones.wrapping_shr(r0).rotate_left(r1);
                    match (random(), random(), random()) {
                        (false, false, false) => lhs |= mask,
                        (false, false, true) => lhs &= mask,
                        (false, true, _) => lhs ^= mask,
                        (true, false, false) => rhs |= mask,
                        (true, false, true) => rhs &= mask,
                        (true, true, _) => rhs ^= mask,
                    }
                    // do the same for alternating ones and zeros
                    let r0: u32 = bit_indexing_mask & random::<u32>();
                    let r1: u32 = bit_indexing_mask & random::<u32>();
                    let mask = alt_ones.wrapping_shr(r0).rotate_left(r1);
                    match (random(), random(), random()) {
                        (false, false, false) => lhs |= mask,
                        (false, false, true) => lhs &= mask,
                        (false, true, _) => lhs ^= mask,
                        (true, false, false) => rhs |= mask,
                        (true, false, true) => rhs &= mask,
                        (true, true, _) => rhs ^= mask,
                    }
                    // This results in excellent fuzzing entropy such as:
                    // lhs: 00101011110101010101010101010000 rhs: 11111111100001111110111111111111
                    // lhs: 01110101000101010100000000000101 rhs: 11111111100001111110111111111111
                    // lhs: 00000000000000000001000000000000 rhs: 11111111100001111110111111111111
                    // lhs: 00000000000000000001000000000000 rhs: 11111111100011011111111111111111
                    // lhs: 00000000000000000010111111100000 rhs: 00000000000000000000101000000000
                    // lhs: 00000000000000000010111111100000 rhs: 10101000000000000000011101101010
                    // lhs: 00000000000000000010000001100000 rhs: 11111101010101000000011101111111
                    // lhs: 10000000000000101010101011101010 rhs: 11111101010101000000011101111000
                    if rhs != 0 {
                        let (quo, rem) = $unsigned_name(lhs,rhs);
                        assert_eq!(lhs, rhs.wrapping_mul(quo).wrapping_add(rem));
                        // `$signed_name` has already been tested plenty, but the sign bit is set
                        // half the time, so this doubles the work of one fuzzing round
                        let (quo, rem) = $signed_name(lhs as $iX,rhs as $iX);
                        assert_eq!(lhs as $iX, (rhs as $iX).wrapping_mul(quo).wrapping_add(rem));
                    }
                }
            }
        )+
    }
}

/// Repeats a block of code `$b`. The number of repeats corresponds to `log2($n) - 1`, where `$n`
/// is the power-of-two literal token. This is intended for unrolling bisection algorithms.
#[allow(unused_macros)]
macro_rules! repeat_log {
    (128, $b:block) => {
        $b;
        repeat_log!(64, $b);
    };
    (64, $b:block) => {
        $b;
        repeat_log!(32, $b);
    };
    (32, $b:block) => {
        $b;
        repeat_log!(16, $b);
    };
    (16, $b:block) => {
        $b;
        repeat_log!(8, $b);
    };
    (8, $b:block) => {
        $b;
        repeat_log!(4, $b);
    };
    (4, $b:block) => {
        $b;
    };
}

/// Unrolls a loop containing a block of code `$b`. `$i` should be a mutable `isize` set to the
/// number of times the block of code should be run. NOTE: the state of `$i` is not guaranteed to be
/// anything during and after execution of the code fed to this macro, so the variable designated
/// by `$i` must not be used by the block of code or any code after the macro.
///
/// This macro called as `unroll!($n, $i, $b);` is equivalent to
/// `
/// // for _ in 0..$i {
/// //     $b;
/// // }
/// `
/// The power-of-two literal token does not effect the logic, but it does effect code size and
/// performance.
#[rustfmt::skip]
macro_rules! unroll {
    (128, $i:ident, $b:block) => {
        // make sure that the type of `$i` coerces to `isize`, to prevent surprises
        let _: isize = $i;
        unroll!(64, $i, $b);
    };
    (64, $i:ident, $b:block) => {
        let _: isize = $i;
        unroll!(32, $i, $b);
    };
    (32, $i:ident, $b:block) => {
        let _: isize = $i;
        // Code gen is almost always too large to unroll 16 times or more. If the block consists of
        // only one or two assembly instructions, it is probably better to code a custom assembly
        // variable jump into an unrolled loop.
        /*
        loop {
            $i -= 16;
            if $i < 0 {
                break;
            }
            $b;$b;$b;$b;$b;$b;$b;$b;$b;$b;$b;$b;$b;$b;$b;$b;
        }
        $i += 16;
        if $i != 0 {
            unroll!(16, $i, $b);
        }
        */
        unroll!(16, $i, $b);
    };
    (16, $i:ident, $b:block) => {
        let _: isize = $i;
        // loop management is kept down to 2 instructions for each loop, plus 2 instructions for
        // every change to a smaller unroll with this method.
        loop {
            $i -= 8;
            if $i < 0 {
                break;
            }
            $b;$b;$b;$b;$b;$b;$b;$b;
        }
        $i += 8;
        // The check for zero is not required, but is a simple way to improve performance
        if $i != 0 {
            unroll!(8, $i, $b);
        }
    };
    (8, $i:ident, $b:block) => {
        let _: isize = $i;
        loop {
            $i -= 4;
            if $i < 0 {
                break;
            }
            $b;$b;$b;$b;
        }
        $i += 4;
        if $i != 0 {
            unroll!(4, $i, $b);
        }
    };
    (4, $i:ident, $b:block) => {
        let _: isize = $i;
        loop {
            $i -= 2;
            if $i < 0 {
                break;
            }
            $b;$b;
        }
        if $i == -1 {
            $b;
        }
    };
    (2, $i:ident, $b:block) => {
        let _: isize = $i;
        loop {
            $i -= 1;
            if $i < 0 {
                break;
            }
            $b;
        }
    };
}

macro_rules! impl_normalization_shift {
    (
        $name:ident, // name of the normalization shift function
        $n:tt, // the number of bits in a $iX or $uX
        $uX:ident, // unsigned integer type for the inputs of `$name`
        $iX:ident, // signed integer type for the inputs of `$name`
        $($unsigned_attr:meta),* // attributes for the function
    ) => {
        /// Finds the shift left that the divisor `div` would need to be normalized for a binary
        /// long division step with the dividend `duo`. This was designed for architectures without
        /// assembly instructions to count the leading zeros of integers.
        ///
        /// NOTE: This function assumes that three edge cases have been handled before reaching it:
        /// `
        /// if div == 0 {
        ///     panic!("attempt to divide by zero")
        /// }
        /// if duo < div {
        ///     return (0, duo)
        /// }
        /// // This eliminates cases where the most significant bit of `div` is set. Signed
        /// // comparisons (for architectures without flags) inside `normalization_shift` and code
        /// // after it can then be used.
        /// if (duo - div) < div {
        ///     return (1, duo - div)
        /// }
        /// `
        $(
            #[$unsigned_attr]
        )*
        fn $name(duo: $uX, div: $uX) -> usize {
            // We have to find the leading zeros of `div` to know where its most significant bit
            // is to even begin binary long division. It is also good to know where the most
            // significant bit of `duo` is so that useful work can be started instead of shifting
            // `div` for all possible quotients (many division steps are wasted if
            // `duo.leading_zeros()` is large and `div` starts out being shifted all the way to the
            // most significant bit). Aligning the most significant bit of `div` and `duo` could be
            // done by shifting `div` left by `div.leading_zeros() - duo.leading_zeros()`, but some
            // CPUs without division hardware also do not have single instructions for calculating
            // `leading_zeros`. Instead of software doing two bisections to find the two
            // `leading_zeros`, we do one bisection to find
            // `div.leading_zeros() - duo.leading_zeros()` without actually knowing either of the
            // leading zeros values.

            // If we shift `duo` right and subtract `div` from the shifted value, and the result is
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
            // The tricky part is when the significant bits are aligned with each other. In that
            // case, `duo.wrapping_sub(div)` could be positive or negative and this algorithm falls
            // into a repeating cycle between two values of `shift` (in this case, it will cycle
            // between shifts of 4 and 5). The smaller of the two shift values turns out to always
            // be valid for starting long division.
            //
            // (the last step uses `level = 1` again instead of `level = 0`)
            // level: 1, shift: 5
            // duo >> shift: 0b00000101
            //          div: 0b00000110
            //             - __________
            //          sub: 0b11111111
            // sub is negative, so decrease the shift, otherwise keep the shift the same.
            /*
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
                // narrow down bisection
                level >>= 1;
                if level == 0 {
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
                    // that assume normalization, but this cannot happen with `sub < 0`.
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
            }
            */

            // Architectures without hardware division or `usize::leading_zeros` support
            #[cfg(any(feature = "force_software_normalization", target_arch = "riscv32i", target_arch = "thumbv6m"))]
            {
                let mut level: usize = $n / 4;
                let mut shift: usize = $n / 2;
                // this macro unrolls the algorithm and compilers can easily propogate constants
                repeat_log!($n, {
                    if (duo >> shift) < div {
                        shift -= level;
                    } else {
                        shift += level;
                    }
                    level >>= 1;
                    if level == 0 {
                        if (duo >> shift) < div {
                            shift -= 1;
                        }
                    }
                });
                shift
            }

            #[cfg(not(any(feature = "force_software_normalization", target_arch = "riscv32i", target_arch = "thumbv6m")))]
            {
                let mut shift = (div.leading_zeros() - duo.leading_zeros()) as usize;
                if duo < (div << shift) {
                    shift -= 1;
                }
                shift
            }
        }
    }
}
