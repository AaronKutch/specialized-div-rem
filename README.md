# Specialized Division and Remainder Algorithms

Currently, this crate provides an alternative form of long division that is faster than the functions provided by Rust primitives on some CPUs.
Note that setting the the flag for compilation to a native cpu can make a significant performance improvement for these functions.

Most division algorithms end up doing most of the work to get both the quotient and remainder, which is why these functions return both (and the compiler can inline and optimize away unused results and calculations).

On naming conventions:
All `_div` functions should really be named `_quo` (quotient) functions, and it would stop the name collision with `div` for divisor, but to keep consistency with `std` it is kept as `_div`.
`duo` is named as such because the variable is kept around and subtracted from inside division functions until it becomes the remainder (so it works as both the dividend and the remainder). This is more apparent when working with allocated big integers (such as with the `apint` crate) that use the input `duo` for intermediate storages and ends up as the remainder.

## Potential Features

Please file an issue or PR if you want to see these or others implemented.

- Other algorithms could be included
- The algorithm could be extended to allow for faster 128 bit division on 32 bit hardware division capable computers, and more.
- Every feature added multiplies the number of functions that must be supplied, so maybe it makes more sense to make only the macros public so that users make only the functions they need.
- The algorithm is actually 3 different algorithms for different sized dividends that could be split up into their own specialized functions. Note: after inspecting assembly used in practice, it seems that the compiler likes to inline the division function with the right branches and returns, so the advantages would be minimal.

## Benchmarks

When running `cargo bench` on this library, it runs division operations 32 times on an array of random numbers.

The names of the benchmarks specify 4 things:

    - the type of integer being operated on,
    - whether the quotient (`_div`) or remainder (`_rem`) or both (`div_rem`) are calculated,
    - the size of the numbers being entered (specifically, how many lower bits the random integer
      is being kept),
    - whether Rust's current algorithm or the algorithm in this crate is being used

For example, the `u128_div_rem_126_64_new` benchmark tests how long it takes to find 32 quotients
and remainders of a u128 random integer with the top 2 bits zeroed divided by a u128 random integer
with the top 64 bits zeroed.

The `constant_u128_div_rem` benchmark is a hardcoded benchmark working on a constant array of integers.
The `_baseline` benchmarks are just there to make sure that the process of adding the 32 answers together (to prevent the compiler from optimizing away code) does not take a lot of time.

The `_new` benches are using the algorithm in this library and the `_std` benches are using the algorithms Rust is using for `/` and `%`.

On an AMD FX-9800P RADEON R7, the benchmarks look like this.
Note that all of the 128 bit benches show improvement of the long division over the one Rust is using.
```
test constant_u128_div_rem_long ... bench:       1,982 ns/iter (+/- 547)
test constant_u128_div_rem_std  ... bench:       4,168 ns/iter (+/- 869)
test i128_div_rem_all_mid_long  ... bench:       1,377 ns/iter (+/- 271)
test i128_div_rem_all_mid_std   ... bench:       5,745 ns/iter (+/- 1,521)
test u128_baseline              ... bench:         154 ns/iter (+/- 100)
test u128_div_all_0_long        ... bench:       3,899 ns/iter (+/- 777)
test u128_div_all_0_std         ... bench:      13,140 ns/iter (+/- 3,072)
test u128_div_all_all_long      ... bench:       1,283 ns/iter (+/- 534)
test u128_div_all_all_std       ... bench:       2,618 ns/iter (+/- 793)
test u128_div_all_mid_long      ... bench:       2,444 ns/iter (+/- 122)
test u128_div_all_mid_std       ... bench:       9,627 ns/iter (+/- 691)
test u128_div_rem_all_0_long    ... bench:       1,744 ns/iter (+/- 196)
test u128_div_rem_all_0_std     ... bench:      15,593 ns/iter (+/- 2,728)
test u128_div_rem_all_all_long  ... bench:         556 ns/iter (+/- 110)
test u128_div_rem_all_all_std   ... bench:       1,596 ns/iter (+/- 421)
test u128_div_rem_all_lo_long   ... bench:       2,299 ns/iter (+/- 518)
test u128_div_rem_all_lo_std    ... bench:       8,691 ns/iter (+/- 1,710)
test u128_div_rem_all_mid_long  ... bench:       1,595 ns/iter (+/- 551)
test u128_div_rem_all_mid_std   ... bench:       5,191 ns/iter (+/- 1,501)
test u128_rem_all_mid_long      ... bench:       1,331 ns/iter (+/- 224)
test u128_rem_all_mid_std       ... bench:       4,928 ns/iter (+/- 933)
(the 64 and 32 bit benches are not included here because the algorithm does not improve on these on this cpu)
```

On an Intel i3-3240,
```
test constant_u128_div_rem_new ... bench:       1,391 ns/iter (+/- 1,486)
test constant_u128_div_rem_std ... bench:       3,128 ns/iter (+/- 439)
test i128_div_rem_128_96_new   ... bench:         944 ns/iter (+/- 306)
test i128_div_rem_128_96_std   ... bench:       3,929 ns/iter (+/- 1,045)
test u128_baseline             ... bench:          67 ns/iter (+/- 0)
test u128_div_128_96_new       ... bench:       1,087 ns/iter (+/- 169)
test u128_div_128_96_std       ... bench:       3,611 ns/iter (+/- 336)
test u128_div_rem_126_64_new   ... bench:       1,245 ns/iter (+/- 583)
test u128_div_rem_126_64_std   ... bench:       6,492 ns/iter (+/- 138)
test u128_div_rem_128_128_new  ... bench:         446 ns/iter (+/- 106)
test u128_div_rem_128_128_std  ... bench:         887 ns/iter (+/- 144)
test u128_div_rem_128_32_new   ... bench:       1,157 ns/iter (+/- 560)
test u128_div_rem_128_32_std   ... bench:       9,582 ns/iter (+/- 889)
test u128_div_rem_128_64_new   ... bench:       1,504 ns/iter (+/- 1,056)
test u128_div_rem_128_64_std   ... bench:       6,587 ns/iter (+/- 173)
test u128_div_rem_128_96_new   ... bench:         993 ns/iter (+/- 1,067)
test u128_div_rem_128_96_std   ... bench:       3,741 ns/iter (+/- 2,889)
test u128_rem_128_96_new       ... bench:       1,017 ns/iter (+/- 1,075)
test u128_rem_128_96_std       ... bench:       3,691 ns/iter (+/- 172)
(the 64 and 32 bit benches are not included here because the algorithm does not improve on these on this cpu)
```
