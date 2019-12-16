# Specialized Division and Remainder Algorithms

This crate provides the algorithms, tests, and benchmarks for three different division functions:

- The `_binary_shift` functions for CPUs without hardware dividers
- The `_trifecta` functions designed for dividing integers larger than the largest hardware division
  a CPU supports
- The `_asymmetric` functions similar to the `_trifecta` functions, except optimized for CPUs with
  an asymmetric sized hardware division function such as x86_64's `divq`

Note that setting the the `asm` feature flag and compiling with `--cpu=native` can cause a
significant performance improvement for these functions, and is absolutely required for
`_asymmetric` to work efficiently.

Most division algorithms end up doing most of the work to get both the quotient and remainder, which is why these functions return both (and the compiler can inline and optimize away unused results and calculations).

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

These benchmarks were run on Rust 1.41.0-nightly (412f43ac5 2019-11-24) with
`set RUSTFLAGS=-C target-cpu=native` and `cargo bench --features=asm`

On an Intel i3-3240, the benchmarks look like this.

```
test i128_div_rem_96_70_asymmetric     ... bench:         644 ns/iter (+/- 12)
test i128_div_rem_96_70_binary_shift   ... bench:       1,937 ns/iter (+/- 123)
test i128_div_rem_96_70_std            ... bench:       3,304 ns/iter (+/- 51)
test i128_div_rem_96_70_trifecta       ... bench:         757 ns/iter (+/- 11)
test u128_div_96_70_asymmetric         ... bench:         623 ns/iter (+/- 9)
test u128_div_96_70_binary_shift       ... bench:       1,890 ns/iter (+/- 53)
test u128_div_96_70_std                ... bench:       3,053 ns/iter (+/- 362)
test u128_div_96_70_trifecta           ... bench:         736 ns/iter (+/- 105)
test u128_div_rem_120_64_asymmetric    ... bench:         979 ns/iter (+/- 99)
test u128_div_rem_120_64_binary_shift  ... bench:       6,288 ns/iter (+/- 415)
test u128_div_rem_120_64_std           ... bench:       5,849 ns/iter (+/- 1,192)
test u128_div_rem_120_64_trifecta      ... bench:       1,384 ns/iter (+/- 71)
test u128_div_rem_128_128_asymmetric   ... bench:         789 ns/iter (+/- 248)
test u128_div_rem_128_128_binary_shift ... bench:         327 ns/iter (+/- 19)
test u128_div_rem_128_128_std          ... bench:         854 ns/iter (+/- 55)
test u128_div_rem_128_128_trifecta     ... bench:         461 ns/iter (+/- 9)
test u128_div_rem_128_56_asymmetric    ... bench:       1,268 ns/iter (+/- 64)
test u128_div_rem_128_56_binary_shift  ... bench:       7,768 ns/iter (+/- 1,032)
test u128_div_rem_128_56_std           ... bench:       7,369 ns/iter (+/- 2,296)
test u128_div_rem_128_56_trifecta      ... bench:       1,892 ns/iter (+/- 251)
test u128_div_rem_128_8_asymmetric     ... bench:         965 ns/iter (+/- 332)
test u128_div_rem_128_8_binary_shift   ... bench:         998 ns/iter (+/- 66)
test u128_div_rem_128_8_std            ... bench:      11,484 ns/iter (+/- 230)
test u128_div_rem_128_8_trifecta       ... bench:       1,153 ns/iter (+/- 32)
test u128_div_rem_128_96_asymmetric    ... bench:         985 ns/iter (+/- 248)
test u128_div_rem_128_96_binary_shift  ... bench:       2,316 ns/iter (+/- 137)
test u128_div_rem_128_96_std           ... bench:       3,657 ns/iter (+/- 1,190)
test u128_div_rem_128_96_trifecta      ... bench:       1,118 ns/iter (+/- 110)
test u128_div_rem_96_32_asymmetric     ... bench:         980 ns/iter (+/- 36)
test u128_div_rem_96_32_binary_shift   ... bench:         939 ns/iter (+/- 274)
test u128_div_rem_96_32_std            ... bench:       6,626 ns/iter (+/- 2,007)
test u128_div_rem_96_32_trifecta       ... bench:       1,117 ns/iter (+/- 63)
test u128_div_rem_96_70_asymmetric     ... bench:         880 ns/iter (+/- 144)
test u128_div_rem_96_70_binary_shift   ... bench:       1,836 ns/iter (+/- 89)
test u128_div_rem_96_70_std            ... bench:       3,170 ns/iter (+/- 126)
test u128_div_rem_96_70_trifecta       ... bench:         761 ns/iter (+/- 60)
test u128_rem_96_70_asymmetric         ... bench:         618 ns/iter (+/- 24)
test u128_rem_96_70_binary_shift       ... bench:       1,885 ns/iter (+/- 141)
test u128_rem_96_70_std                ... bench:       3,042 ns/iter (+/- 93)
test u128_rem_96_70_trifecta           ... bench:         738 ns/iter (+/- 23)
(the 64 and 32 bit benches are not included here because the algorithms are not faster than the
native divisions)
```

On an AMD FX-9800P RADEON R7

```
test i128_div_rem_96_70_asymmetric     ... bench:         707 ns/iter (+/- 167)
test i128_div_rem_96_70_binary_shift   ... bench:       3,997 ns/iter (+/- 598)
test i128_div_rem_96_70_std            ... bench:       4,639 ns/iter (+/- 319)
test i128_div_rem_96_70_trifecta       ... bench:         796 ns/iter (+/- 157)
test u128_div_96_70_asymmetric         ... bench:         665 ns/iter (+/- 198)
test u128_div_96_70_binary_shift       ... bench:       3,849 ns/iter (+/- 541)
test u128_div_96_70_std                ... bench:       4,243 ns/iter (+/- 1,158)
test u128_div_96_70_trifecta           ... bench:         790 ns/iter (+/- 214)
test u128_div_rem_120_64_asymmetric    ... bench:         605 ns/iter (+/- 124)
test u128_div_rem_120_64_binary_shift  ... bench:       6,995 ns/iter (+/- 1,018)
test u128_div_rem_120_64_std           ... bench:       7,620 ns/iter (+/- 1,746)
test u128_div_rem_120_64_trifecta      ... bench:       1,460 ns/iter (+/- 300)
test u128_div_rem_128_128_asymmetric   ... bench:         580 ns/iter (+/- 69)
test u128_div_rem_128_128_binary_shift ... bench:         464 ns/iter (+/- 35)
test u128_div_rem_128_128_std          ... bench:       1,215 ns/iter (+/- 196)
test u128_div_rem_128_128_trifecta     ... bench:         463 ns/iter (+/- 46)
test u128_div_rem_128_56_asymmetric    ... bench:         870 ns/iter (+/- 200)
test u128_div_rem_128_56_binary_shift  ... bench:       7,231 ns/iter (+/- 1,769)
test u128_div_rem_128_56_std           ... bench:       9,566 ns/iter (+/- 1,671)
test u128_div_rem_128_56_trifecta      ... bench:       4,662 ns/iter (+/- 80)
test u128_div_rem_128_8_asymmetric     ... bench:       3,542 ns/iter (+/- 58)
test u128_div_rem_128_8_binary_shift   ... bench:       3,545 ns/iter (+/- 129)
test u128_div_rem_128_8_std            ... bench:      36,875 ns/iter (+/- 538)
test u128_div_rem_128_8_trifecta       ... bench:       3,566 ns/iter (+/- 61)
test u128_div_rem_128_96_asymmetric    ... bench:       1,788 ns/iter (+/- 152)
test u128_div_rem_128_96_binary_shift  ... bench:       9,925 ns/iter (+/- 417)
test u128_div_rem_128_96_std           ... bench:      13,095 ns/iter (+/- 733)
test u128_div_rem_128_96_trifecta      ... bench:       2,660 ns/iter (+/- 97)
test u128_div_rem_96_32_asymmetric     ... bench:       2,007 ns/iter (+/- 154)
test u128_div_rem_96_32_binary_shift   ... bench:       2,376 ns/iter (+/- 120)
test u128_div_rem_96_32_std            ... bench:      21,713 ns/iter (+/- 615)
test u128_div_rem_96_32_trifecta       ... bench:       2,409 ns/iter (+/- 143)
test u128_div_rem_96_70_asymmetric     ... bench:       1,360 ns/iter (+/- 146)
test u128_div_rem_96_70_binary_shift   ... bench:       7,897 ns/iter (+/- 763)
test u128_div_rem_96_70_std            ... bench:      11,181 ns/iter (+/- 223)
test u128_div_rem_96_70_trifecta       ... bench:       1,405 ns/iter (+/- 175)
test u128_rem_96_70_asymmetric         ... bench:       1,056 ns/iter (+/- 61)
test u128_rem_96_70_binary_shift       ... bench:       3,857 ns/iter (+/- 963)
test u128_rem_96_70_std                ... bench:       4,252 ns/iter (+/- 730)
test u128_rem_96_70_trifecta           ... bench:         764 ns/iter (+/- 229)
(the 64 and 32 bit benches are not included here because the algorithms are not faster than the
native divisions)
```
