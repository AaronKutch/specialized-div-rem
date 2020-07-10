#[cfg(test)]
use rand::random;

/// Returns the number of leading binary zeros in `x`.
///
/// Counting leading zeros is performance critical to certain division algorithms and more. This is
/// a fast software routine for computer architectures without an assembly instruction to quickly
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
