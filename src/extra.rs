//! When copying and pasting to `compiler-builtins`, we want to minimize changes to any files
//! outside of `lib.rs` and this file. This file contains functions and macros that might be used by
//! `compiler-builtins`, but would need to be selectively pasted to separate places.

#[cfg(test)]
use rand::random;

/// Returns the number of leading binary zeros in `x`.
///
/// Counting leading zeros is performance critical to certain division algorithms and more. This is
/// a fast software routine for CPU architectures without an assembly instruction to quickly
/// calculate `leading_zeros`.
pub const fn usize_leading_zeros(x: usize) -> usize {
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
    // boolean expressions into an integer (performed on many architecture with a set-if-not-equal
    // instruction).

    // Adapted from LLVM's `compiler-rt/lib/builtins/clzsi2.c`. The original sets the number of
    // zeros `z` to be 0 and adds to that:
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
    // instead. Architectures with a set-if-not-equal instruction probably also have a
    // set-if-more-than-or-equal instruction, so we use that. If the architecture does not have such
    // and instruction and has to branch, this is still probably the fastest method:
    //
    // // use `>=` comparison instead
    // let t = ((x >= const) as usize) << level;
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
        t = ((x >= (1 << 32)) as usize) << 5;
        x >>= t;
        z -= t;
    }

    #[cfg(any(target_pointer_width = "32", target_pointer_width = "64"))]
    {
        t = ((x >= (1 << 16)) as usize) << 4;
        x >>= t;
        z -= t;
    }

    t = ((x >= (1 << 8)) as usize) << 3;
    x >>= t;
    z -= t;

    t = ((x >= (1 << 4)) as usize) << 2;
    x >>= t;
    z -= t;

    t = ((x >= (1 << 2)) as usize) << 1;
    x >>= t;
    z -= t;

    t = (x >= (1 << 1)) as usize;
    x >>= t;
    z -= t;

    // All bits except LSB are guaranteed to be zero by this point. If `x != 0` then `x == 1` and
    // subtracts a potential zero from `z`.
    z - x

    // We could potentially save a few cycles by using the LUT trick from
    // "https://embeddedgurus.com/state-space/2014/09/
    // fast-deterministic-and-portable-counting-leading-zeros/". 256 bytes for a LUT is too large,
    // so we could perform bisection down to `((x >= (1 << 4)) as usize) << 2` and use this 16 byte
    // LUT for the rest of the work:
    //const LUT: [u8; 16] = [0, 1, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 4, 4, 4, 4];
    //z -= LUT[x] as usize;
    //z
    // However, it ends up generating about the same number of instructions. When benchmarked on
    // x86_64, it is slightly faster to use the LUT, but this is probably because of OOO execution
    // effects. Changing to using a LUT and branching is risky for smaller cores.
}

#[test]
fn usize_leading_zeros_test() {
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
        if usize_leading_zeros(x) != (x.leading_zeros() as usize) {
            panic!(
                "x: {}, expected: {}, found: {}",
                x,
                x.leading_zeros(),
                usize_leading_zeros(x)
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
        $($test_name:ident, $unsigned_name:ident, $signed_name:ident);+;
    ) => {
        $(
            #[test]
            fn $test_name() {
                fn assert_invariants(lhs: $uX, rhs: $uX) {
                    let (quo, rem) = $unsigned_name(lhs, rhs);
                    if rhs <= rem || (lhs != rhs.wrapping_mul(quo).wrapping_add(rem)) {
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
                    // We cannot just test that
                    // `lhs == rhs.wrapping_mul(quo).wrapping_add(rem)`, but also
                    // need to make sure the remainder isn't larger than the divisor
                    // and has the correct sign.
                    let incorrect_rem = if rem == 0 {
                        false
                    } else if rhs == $iX::MIN {
                        // `rhs.wrapping_abs()` would overflow, so handle this case
                        // separately.
                        (lhs.is_negative() != rem.is_negative()) || (rem == $iX::MIN)
                    } else {
                        (lhs.is_negative() != rem.is_negative())
                        || (rhs.wrapping_abs() <= rem.wrapping_abs())
                    };
                    if incorrect_rem || lhs != rhs.wrapping_mul(quo).wrapping_add(rem) {
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
                                    assert_invariants(lhs, rhs);
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
                    // Randomly OR, AND, and XOR randomly sized and shifted continuous strings of
                    // ones with `lhs` and `rhs`. XOR is performed most often because OR and AND
                    // tend to be destructive. This results in excellent fuzzing entropy such as:
                    // lhs: 00101011110101010101010101010000 rhs: 11111111100001111110111111111111
                    // lhs: 01110101000101010100000000000101 rhs: 11111111100001111110111111111111
                    // lhs: 00000000000000000001000000000000 rhs: 11111111100001111110111111111111
                    // lhs: 00000000000000000001000000000000 rhs: 11111111100011011111111111111111
                    // lhs: 00000000000000000010111111100000 rhs: 00000000000000000000101000000000
                    // lhs: 00000000000000000010111111100000 rhs: 10101000000000000000011101101010
                    // lhs: 00000000000000000010000001100000 rhs: 11111101010101000000011101111111
                    // lhs: 10000000000000101010101011101010 rhs: 11111101010101000000011101111000
                    // The msb is set half of the time by the fuzzer, but `assert_invariants` tests
                    // both the signed and unsigned functions.
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

                    if rhs != 0 {
                        assert_invariants(lhs, rhs);
                    }
                }
            }
        )+
    }
}
