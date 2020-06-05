# Specialized Division and Remainder Algorithms

This crate is not intended for direct use, but for use in parts of compilers (such as
`compiler-builtins`), so that all division code can benefit.

This crate provides the algorithms, tests, and benchmarks for four different division functions:

- The `_binary_long` functions for CPUs without hardware dividers
- The `_delegate` functions similar to `_binary_long`, but with calls to smaller divisions if
  possible
- The `_trifecta` functions designed for dividing integers larger than the largest hardware division
  a CPU supports. These become efficient for 128 bit divisions, for both CPUs with and without
  hardware dividers. Note that this function depends upon fast multpliers, such that `_delegate` can
  outperform this function even with hardware dividers in some cases.
- The `_asymmetric` functions similar to the `_trifecta` functions, except optimized for CPUs with
  an asymmetric sized hardware division function such as x86_64's division instruction

Note that setting the the `asm` feature flag can cause a significant performance improvement for
these functions, and is absolutely required for `_asymmetric` to work efficiently. The `std` flag is
only needed for benchmarks and tests.

Most division algorithms end up doing most of the work to get both the quotient and remainder, which
is why these functions return both (and the compiler can inline and optimize away unused results and
calculations).

On naming conventions:
All `_div` functions should really be named `_quo` (quotient) functions, and it would stop the name
collision with `div` for divisor, but to keep consistency with `std` it is kept as `_div`.
`duo` is named as such to avoid the collision between the "div" in dividend and divisor, and because
in many algorithms it is kept around and subtracted from inside division functions until it becomes
the remainder (so it works as both the dividend and the remainder).

## Benchmarks

When running `cargo bench` on this library, it runs division operations 32 times on an array of
random numbers masked to benchmark different ranges of dividends and divisors.

The names of the benchmarks specify 4 things:

    - the type of integer being operated on
    - whether the quotient (`_div`) or remainder (`_rem`) or both (`div_rem`) are calculated
    - the size of the numbers being entered (specifically, how many lower bits of the random integer
      are being kept)
    - the kind of algorithm. Whatever Rust's `/` and `%` operators are using is benchmarked by
      the `_std` benches.

For example, the `u128_div_rem_126_64_asymmetric` benchmark tests how long it takes to find 32
quotients and remainders of a u128 random integer with the top 128 - 126 = 2 bits zeroed, divided
by a u128 random integer with the top 128 - 64 = 64 bits zeroed, using the asymmetric algorithm.

On an Intel i3-3240, the benchmarks look like this. This benchmark was run on Rust 1.41.0-nightly (412f43ac5 2019-11-24) with
`set RUSTFLAGS=-C target-cpu=native` and `cargo bench --features=asm`:

```
test i128_div_rem_96_32_asymmetric    ... bench:       1,008 ns/iter (+/- 21)
test i128_div_rem_96_32_binary_long   ... bench:       4,887 ns/iter (+/- 1,355)
test i128_div_rem_96_32_delegate      ... bench:         977 ns/iter (+/- 48)
test i128_div_rem_96_32_std           ... bench:       6,739 ns/iter (+/- 2,184)
test i128_div_rem_96_32_trifecta      ... bench:       1,088 ns/iter (+/- 170)
test u128_div_rem_120_120_asymmetric  ... bench:         849 ns/iter (+/- 13)
test u128_div_rem_120_120_binary_long ... bench:         301 ns/iter (+/- 76)
test u128_div_rem_120_120_delegate    ... bench:         344 ns/iter (+/- 233)
test u128_div_rem_120_120_std         ... bench:         808 ns/iter (+/- 22)
test u128_div_rem_120_120_trifecta    ... bench:         469 ns/iter (+/- 241)
test u128_div_rem_128_64_asymmetric   ... bench:       1,082 ns/iter (+/- 243)
test u128_div_rem_128_64_delegate     ... bench:       7,013 ns/iter (+/- 330)
test u128_div_rem_128_64_std          ... bench:       6,517 ns/iter (+/- 260)
test u128_div_rem_128_64_trifecta     ... bench:       1,677 ns/iter (+/- 69)
test u128_div_rem_128_8_asymmetric    ... bench:         987 ns/iter (+/- 44)
test u128_div_rem_128_8_binary_long   ... bench:       8,761 ns/iter (+/- 282)
test u128_div_rem_128_8_delegate      ... bench:         997 ns/iter (+/- 52)
test u128_div_rem_128_8_std           ... bench:      11,070 ns/iter (+/- 1,262)
test u128_div_rem_128_8_trifecta      ... bench:       1,160 ns/iter (+/- 324)
test u128_div_rem_128_96_asymmetric   ... bench:       1,073 ns/iter (+/- 64)
test u128_div_rem_128_96_binary_long  ... bench:       2,599 ns/iter (+/- 63)
test u128_div_rem_128_96_delegate     ... bench:       2,345 ns/iter (+/- 154)
test u128_div_rem_128_96_std          ... bench:       3,683 ns/iter (+/- 105)
test u128_div_rem_128_96_trifecta     ... bench:       1,154 ns/iter (+/- 444)
test u128_div_rem_96_32_asymmetric    ... bench:       1,004 ns/iter (+/- 236)
test u128_div_rem_96_32_binary_long   ... bench:       4,902 ns/iter (+/- 510)
test u128_div_rem_96_32_delegate      ... bench:         952 ns/iter (+/- 144)
test u128_div_rem_96_32_std           ... bench:       6,935 ns/iter (+/- 2,460)
test u128_div_rem_96_32_trifecta      ... bench:       1,111 ns/iter (+/- 322)
test u128_div_rem_96_70_asymmetric    ... bench:         640 ns/iter (+/- 98)
test u128_div_rem_96_70_binary_long   ... bench:       2,189 ns/iter (+/- 103)
test u128_div_rem_96_70_delegate      ... bench:       1,876 ns/iter (+/- 1,584)
test u128_div_rem_96_70_std           ... bench:       3,113 ns/iter (+/- 113)
test u128_div_rem_96_70_trifecta      ... bench:         726 ns/iter (+/- 319)
(the rest of the benchmarks are not included here because the algorithms are all not faster than the
hardware divisions)
```
