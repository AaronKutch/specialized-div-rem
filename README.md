# Specialized Division and Remainder Algorithms

This crate is not intended for direct use, but for use in parts of compilers (such as
`compiler-builtins`), so that all division code can benefit. However, this crate might find use
for cases where control over inlining is needed (e.g. see the `u128_div_asymmetric` function which
uses inlining to remove instructions only needed for calculating the remainder).

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

Without any default features on, this crate is in `no_std` mode and only exports macros. When the
`implement` and `std` flags are on, this crate uses its macros to implement a wide arrangement of
division functions for usage in tests and benchmarks. Note that setting the the `asm` feature flag
is absolutely required for `_asymmetric` to work efficiently.

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

When running `cargo bench` on this library with default features, it runs division operations on
random numbers masked to benchmark different ranges of dividends and divisors.

The names of the benchmarks specify 4 things:

    - the type of integer being operated on
    - the size of the numbers being entered (specifically, how many lower bits of the random integer
      are being kept)
    - the kind of algorithm. Whatever Rust's `/` and `%` operators are using is benchmarked by
      the `_std` benches.

For example, the `u128_div_rem_96_70_asymmetric` benchmark tests how long it takes to find the
quotients and remainders of i128 random integers with the top 128 - 96 = 32 bits zeroed, divided
by a u128 random integer with the top 128 - 70 = 58 bits zeroed, using the asymmetric algorithm.

On an Intel i3-3240, the benchmarks look like this. This benchmark was run on Rust 1.46.0-nightly
(8ac1525e0 2020-07-07) with default features:

```
test i128_div_rem_96_32_asymmetric   ... bench:          29 ns/iter (+/- 0)
test i128_div_rem_96_32_delegate     ... bench:          32 ns/iter (+/- 5)
test i128_div_rem_96_32_std          ... bench:         203 ns/iter (+/- 3)
test i128_div_rem_96_32_trifecta     ... bench:          33 ns/iter (+/- 0)
test u128_div_rem_120_120_asymmetric ... bench:          21 ns/iter (+/- 0)
test u128_div_rem_120_120_delegate   ... bench:          16 ns/iter (+/- 0)
test u128_div_rem_120_120_std        ... bench:          24 ns/iter (+/- 2)
test u128_div_rem_120_120_trifecta   ... bench:          14 ns/iter (+/- 0)
test u128_div_rem_128_64_asymmetric  ... bench:          37 ns/iter (+/- 1)
test u128_div_rem_128_64_delegate    ... bench:          86 ns/iter (+/- 7)
test u128_div_rem_128_64_std         ... bench:         218 ns/iter (+/- 62)
test u128_div_rem_128_64_trifecta    ... bench:          61 ns/iter (+/- 1)
test u128_div_rem_128_8_asymmetric   ... bench:          30 ns/iter (+/- 0)
test u128_div_rem_128_8_delegate     ... bench:          31 ns/iter (+/- 2)
test u128_div_rem_128_8_std          ... bench:         371 ns/iter (+/- 2)
test u128_div_rem_128_8_trifecta     ... bench:          34 ns/iter (+/- 0)
test u128_div_rem_128_96_asymmetric  ... bench:          41 ns/iter (+/- 0)
test u128_div_rem_128_96_delegate    ... bench:          55 ns/iter (+/- 4)
test u128_div_rem_128_96_std         ... bench:         119 ns/iter (+/- 0)
test u128_div_rem_128_96_trifecta    ... bench:          43 ns/iter (+/- 1)
test u128_div_rem_96_32_asymmetric   ... bench:          27 ns/iter (+/- 0)
test u128_div_rem_96_32_delegate     ... bench:          54 ns/iter (+/- 1)
test u128_div_rem_96_32_std          ... bench:         212 ns/iter (+/- 2)
test u128_div_rem_96_32_trifecta     ... bench:          33 ns/iter (+/- 0)
test u128_div_rem_96_70_asymmetric   ... bench:          21 ns/iter (+/- 0)
test u128_div_rem_96_70_delegate     ... bench:          46 ns/iter (+/- 0)
test u128_div_rem_96_70_std          ... bench:          97 ns/iter (+/- 0)
test u128_div_rem_96_70_trifecta     ... bench:          24 ns/iter (+/- 0)
(the rest of the benchmarks are not included here, because the 64 bit hardware divisions are always
faster than the algorithms)
```
