//! Each function name has:
//!     - the type it operates on
//!     - if it returns the quotient (`div`), remainder (`rem`), or both (`div_rem`, returns a tuple of the quotient and the remainder) of `duo` divided by `rem`
//!     - `_long` specifying it uses the special long division algorithm
//!     - `_inline_always` specifying that the function has the `#[inline(always)]` attribute on it. Note: these are very useful for getting a little more performance or whenever there is a wrapper using the functions.

mod all_all;

pub use all_all::*;