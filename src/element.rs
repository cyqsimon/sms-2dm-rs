use std::{fmt, str::FromStr};

use num_traits::Unsigned;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{DefaultUnsigned, Error};

/// Convenience macro for defining an element.
macro_rules! mk_el {
    ($(#[$attr:meta])* $name: ident, $n: literal) => {
        $(#[$attr])*
        #[derive(Copy, Clone, Debug, PartialEq)]
        #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
        pub struct $name<U = DefaultUnsigned>
        where
            U: Unsigned + Copy + fmt::Debug,
            Error: From<U::FromStrRadixErr>,
        {
            /// The ID of the element.
            pub id: U,
            /// The IDs of nodes in the element.
            pub nodes: [U; $n],
            /// The ID of the material assigned to the element.
            pub material: U,
        }
        impl<U> FromStr for $name<U>
        where
            U: Unsigned + Copy + fmt::Debug,
            Error: From<U::FromStrRadixErr>,
        {
            type Err = Error;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let mut field_it = s.split_whitespace();

                let Some(stringify!($name)) = field_it.next() else {
                    panic!(r#"{el} tag should be "{el}"."#, el = stringify!($name));
                };

                let id_raw = field_it.next().ok_or(Error::MissingValue)?;
                let id = U::from_str_radix(id_raw, 10)?;

                let mut nodes = [U::zero(); $n];
                for i in 0..$n {
                    let Some(node_raw) = field_it.next() else {
                        Err(Error::MissingValue)?
                    };

                    let node = U::from_str_radix(node_raw, 10)?;
                    nodes[i] = node;
                }

                let mat_raw = field_it.next().ok_or(Error::MissingValue)?;
                let material = U::from_str_radix(mat_raw, 10)?;

                if let Some(v) = field_it.next() {
                    Err(Error::ExtraneousValue(v.into()))?;
                }

                Ok(Self{ id, nodes, material })
            }
        }
    };
}

mk_el! {
    /// Identifies a 2-noded linear element.
    E2L, 2
}
mk_el! {
    /// Identifies a 3-noded linear element.
    E3L, 3
}
mk_el! {
    /// Identifies a 3-noded triangular element.
    E3T, 3
}
mk_el! {
    /// Identifies a 6-noded triangular element.
    E6T, 6
}
mk_el! {
    /// Identifies a 4-noded quadrilateral element.
    E4Q, 4
}
mk_el! {
    /// Identifies a 8-noded quadrilateral element.
    E8Q, 8
}
mk_el! {
    /// Identifies a 9-noded quadrilateral element.
    E9Q, 9
}
