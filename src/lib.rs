// NOTE: ranges (like `0..=x`) should not be used in the algorithms of this library, since they can
// generate references to `memcpy` in unoptimized code. this code is intended to be used by
// `compiler-builtins` which cannot use `memcpy`.

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(feature = "asm", feature(asm))]

#[macro_use]
mod test;

#[macro_use]
mod norm_shift;

#[macro_use]
mod binary_long;

#[macro_use]
mod delegate;

#[macro_use]
mod trifecta;

#[macro_use]
mod asymmetric;

#[cfg(feature = "implement")]
mod implement;
#[cfg(feature = "implement")]
pub use implement::*;
