**Specialized Division and Remainder Algorithms**

Currently, this crate provides an alternative form of long division that is faster than the functions provided by `std` in some cases.
Note that setting the the flag for compilation to a native cpu and using LTO can make a decent performance improvement for these functions.

## Potential Features
Please file an issue or PR if you want to see these or others implemented.
- Other algorithms could be included
- The algorithm could be extended to allow for faster 128 bit division on 32 bit hardware division capable computers, and more.
- The algorithm is actually 3 different algorithms for different sized dividends that could be split up into their own specialized functions.
- Every feature added multiplies the number of functions that must be supplied, so maybe it makes more sense to make only the macros public so that users make only the functions they need.

## Benchmarks

When running `cargo bench` on this library, it runs division operations 1024 times on an array of random numbers.

The names of the benchmarks specify 4 things: 

    - the type of integer being operated on,
    - whether the quotient (`_div`) or remainder (`_rem`) or both (`div_rem`) are calculated,
    - the size of the numbers being entered,
    - whether Rust's current algorithm or the algorithm in this crate is being used

Benchmarks vary based on the numerical size of the numbers being entered, and for each division function 3 different sizes are entered:

    - `_all_all` means that all of the bits of `duo` and `div` are random
    - `_all_mid` means that all the bits of `duo` are random and the lower 3/4 bits of `div` are random (the higher 1/4 of the bits are zero),
    - `_all_0` means that all the bits of `duo` are random and the lower 1/4 bits of `div` are random

The `_long` benches are using the algorithm in this library and the `_std` benches are using the algorithms Rust is using for `/` and `%`.

Additionally, when the benchmarks are run, some of the time per iteration is taken up by operations other than the division operation.
The `_baseline` benchmarks approximate this time.

On an AMD FX-9800P RADEON R7, the benchmarks look like this.
Note that all of the 128 bit benches show improvement of the long division over the one Rust is using.

    test constant_u128_div_rem_long ... bench:       1,507 ns/iter (+/- 373)
    test constant_u128_div_rem_std  ... bench:       4,650 ns/iter (+/- 156)
    test i128_div_rem_all_mid_long  ... bench:      42,608 ns/iter (+/- 6,458)
    test i128_div_rem_all_mid_std   ... bench:     166,380 ns/iter (+/- 4,349)
    test u128_baseline              ... bench:       1,127 ns/iter (+/- 48)
    test u128_div_all_0_long        ... bench:      41,588 ns/iter (+/- 2,333)
    test u128_div_all_0_std         ... bench:     361,722 ns/iter (+/- 47,961)
    test u128_div_all_all_long      ... bench:      13,466 ns/iter (+/- 1,362)
    test u128_div_all_all_std       ... bench:      41,006 ns/iter (+/- 1,606)
    test u128_div_all_mid_long      ... bench:      41,044 ns/iter (+/- 2,594)
    test u128_div_all_mid_std       ... bench:     150,172 ns/iter (+/- 7,004)
    test u128_div_rem_all_0_long    ... bench:      42,894 ns/iter (+/- 2,954)
    test u128_div_rem_all_0_std     ... bench:     368,780 ns/iter (+/- 48,235)
    test u128_div_rem_all_all_long  ... bench:      14,392 ns/iter (+/- 713)
    test u128_div_rem_all_all_std   ... bench:      47,686 ns/iter (+/- 3,233)
    test u128_div_rem_all_mid_long  ... bench:     105,274 ns/iter (+/- 978)
    test u128_div_rem_all_mid_std   ... bench:     390,629 ns/iter (+/- 8,083)
    test u128_rem_all_mid_long      ... bench:     104,386 ns/iter (+/- 3,699)
    test u128_rem_all_mid_std       ... bench:     369,806 ns/iter (+/- 10,855)
    (the 64 and 32 bit benches are not included here because the algorithm does not improve on these on this cpu)