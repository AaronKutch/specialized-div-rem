**Specialized Division and Remainder Algorithms**
Currently, only one alternative form of long division is provided in this crate that is faster than the functions provided by `std` in some cases.

## Potential Features
Please file an issue or PR if you want to see these or others implemented.
- Signed division
- The algorithm could be extended to allow for faster 128 bit division on 32 bit hardware division capable computers, and more.
- The algorithm has 3 different sections for different sized dividends that could be split up into their own specialized functions.
- Every feature added multiplies the number of functions that must be supplied, so maybe it makes more sense to make the macros public so that users only make the functions they need.
- Other algorithms could be included

## Performance
Setting the the flag for compilation to a native cpu makes a big difference for these functions, probably due to the use of count leading zeros instructions.
On windows, run `set RUSTFLAGS= -C target-cpu=native` before `cargo bench`.
When running `cargo bench` on this library, it runs division operations 1024 times on an array of random numbers.
The `_long` benches are using the algorithm in this library and the `_std` benches are using the algorithms Rust is using for `/` and `%`.
The `all_mid` stuff just specifies that all bits of `duo` are randomized and the least significant 3/4 bits of `div` are randomized, and the rest of the bits are 0 (`all` is 4/4 bits, `mid` is 3/4 bits, `lo` is 1/2 bits, and `0` is the lowest quarter of the input type).
Additionally, when the benchmarks are run, some of the time per iteration is taken up by operations other than the division operation.
The `_baseline` benchmarks approximate this time.

On an AMD FX-9800P RADEON R7, the benchmarks look like this.
Note that the u128_div_inline_always_all_lo_std benchmark (the most important one for me) takes 4.6x as much time as the long division one, and all of the other 128 benches show some kind of improvement.
test constant_u128_div_rem_long              ... bench:       1,502 ns/iter (+/- 222)
test constant_u128_div_rem_std               ... bench:       3,848 ns/iter (+/- 237)
test u128_baseline                           ... bench:       1,102 ns/iter (+/- 116)
test u128_div_inline_always_all_lo_long      ... bench:      55,887 ns/iter (+/- 7,532)
test u128_div_inline_always_all_lo_std       ... bench:     254,244 ns/iter (+/- 10,869)
test u128_div_rem_0_0_long                   ... bench:      18,000 ns/iter (+/- 1,204)
test u128_div_rem_0_0_std                    ... bench:      27,575 ns/iter (+/- 2,728)
test u128_div_rem_0_all_long                 ... bench:       9,470 ns/iter (+/- 686)
test u128_div_rem_0_all_std                  ... bench:      26,247 ns/iter (+/- 4,238)
test u128_div_rem_all_0_long                 ... bench:      42,620 ns/iter (+/- 2,215)
test u128_div_rem_all_0_std                  ... bench:     360,801 ns/iter (+/- 28,081)
test u128_div_rem_all_all_long               ... bench:      14,782 ns/iter (+/- 648)
test u128_div_rem_all_all_std                ... bench:      48,359 ns/iter (+/- 4,339)
test u128_div_rem_all_mid_long               ... bench:      42,472 ns/iter (+/- 3,815)
test u128_div_rem_all_mid_std                ... bench:     158,903 ns/iter (+/- 18,172)
test u128_div_rem_inline_always_all_lo_long  ... bench:      56,618 ns/iter (+/- 7,046)
test u128_div_rem_inline_always_all_lo_std   ... bench:     256,461 ns/iter (+/- 44,215)
test u128_div_rem_inline_always_all_mid_long ... bench:      37,072 ns/iter (+/- 4,686)
test u128_div_rem_inline_always_all_mid_std  ... bench:     157,801 ns/iter (+/- 13,818)
test u128_div_rem_lo_0_long                  ... bench:      26,059 ns/iter (+/- 1,808)
test u128_div_rem_lo_0_std                   ... bench:      36,102 ns/iter (+/- 2,342)
test u128_rem_inline_always_all_lo_long      ... bench:      54,026 ns/iter (+/- 3,503)
test u128_rem_inline_always_all_lo_std       ... bench:     243,219 ns/iter (+/- 40,856)
(the 64 and 32 bit benches are not included here because the algorithm does not improve on these on this cpu)

It is unknown at this time if the 32 bit long division algorithms outperform binary long division on 16 bit computers (and likewise the 64 bit ones). Please message the maintainer about the results of the benchmarks if you run them on computers from classes not included above. The algorithm's efficiency decreases with hardware division size and I am curious where the breakeven point is.