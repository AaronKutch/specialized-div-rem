macro_rules! impl_binary_long {
    (
        $unsigned_name:ident, // name of the unsigned function
        $signed_name:ident, // name of the signed function
        $n:expr, // the number of bits in a $iX or $uX
        $uX:ident, // unsigned integer that will be shifted
        $iX:ident, // signed version of $uX
        $($unsigned_attr:meta),*; // attributes for the unsigned function
        $($signed_attr:meta),* // attributes for the signed function
    ) => {

        /// Computes the quotient and remainder of `duo` divided by `div` and returns them as a
        /// tuple.
        /// 
        /// This uses binary shift long division only.
        ///
        /// # Panics
        ///
        /// When attempting to divide by zero, this function will panic.
        $(
            #[$unsigned_attr]
        )*
        pub fn $unsigned_name(duo: $uX, div: $uX) -> ($uX,$uX) {
            if div == 0 {
                panic!("division by zero")
            }

            // Full $uX binary long division. Use `leading_zeros` on the first round,
            // because we assume that the average usage of division has arguments that
            // are random but have a significant number of leading zero bits. Doing
            // `leading_zeros` for every round would be very expensive, especially for
            // CPUs without a native count leading zeros instruction, but doing it just
            // for the first round is advantageous for both performance of the common
            // case and for code simplicity. Note that many benchmarks use the full
            // `n_d` bits for `duo`, and the benchmarks with several bits less have a
            // good performance increase.

            let div_lz = div.leading_zeros();
            let duo_lz = duo.leading_zeros();

            if div_lz < duo_lz {
                return (0, duo)
            }

            // Figures out how far `div` should be shifted to align most significant
            // bits
            let mut shift = div_lz - duo_lz;
            let mut duo = duo;
            let mut div = div << shift;
            let mut quo = 0;
            loop {
                // There is a way to do this without branching, but requires too many extra
                // operations to be faster:
                // let sub = duo.wrapping_sub(div);
                // let sign_mask = !(((sub as $iD) >> (n_d - 1)) as $uD);
                // duo -= div & sign_mask;
                // quo |= sign_mask & 1;
                let sub = duo.wrapping_sub(div);
                if (sub as $iX) >= 0 {
                    duo = sub;
                    quo |= 1;
                }

                if duo == 0 {
                    return (quo << shift, duo)
                }
                if shift == 0 {
                    return (quo, duo)
                }
                shift -= 1;
                div >>= 1;
                quo <<= 1;
            }
        }

        /// Computes the quotient and remainder of `duo` divided by `div` and returns them as a
        /// tuple.
        /// 
        /// This uses binary shift long division only.
        ///
        /// # Panics
        ///
        /// When attempting to divide by zero, this function will panic.
        $(
            #[$signed_attr]
        )*
        pub fn $signed_name(duo: $iX, div: $iX) -> ($iX,$iX) {
            match (duo < 0, div < 0) {
                (false,false) => {
                    let t = $unsigned_name(duo as $uX,div as $uX);
                    (t.0 as $iX,t.1 as $iX)
                },
                (true,false) => {
                    let t = $unsigned_name(duo.wrapping_neg() as $uX,div as $uX);
                    ((t.0 as $iX).wrapping_neg(),(t.1 as $iX).wrapping_neg())
                },
                (false,true) => {
                    let t = $unsigned_name(duo as $uX,div.wrapping_neg() as $uX);
                    ((t.0 as $iX).wrapping_neg(),t.1 as $iX)
                },
                (true,true) => {
                    let t = $unsigned_name(duo.wrapping_neg() as $uX,div.wrapping_neg() as $uX);
                    (t.0 as $iX,(t.1 as $iX).wrapping_neg())
                },
            }
        }
    }
}
