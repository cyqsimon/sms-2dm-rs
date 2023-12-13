use std::{fmt, str::FromStr};

use num_traits::{Float, Unsigned};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{error::weak_error, DefaultFloat, DefaultUnsigned, Error};

pub(crate) const NODE_TAG: &str = "ND";

/// Defines the ID and location for each node of the mesh.
///
/// Corresponds to the card `ND`.
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Node<U = DefaultUnsigned, F = DefaultFloat>
where
    U: Unsigned + Copy + fmt::Debug,
    F: Float + fmt::Debug,
    Error: From<U::FromStrRadixErr> + From<F::FromStrRadixErr>,
{
    /// The ID of the node.
    pub id: U,
    /// The x, y, and z coordinates of the point.
    pub coordinate: [F; 3],
}
impl<U, F> FromStr for Node<U, F>
where
    U: Unsigned + Copy + fmt::Debug,
    F: Float + fmt::Debug,
    Error: From<U::FromStrRadixErr> + From<F::FromStrRadixErr>,
{
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut field_it = s.split_whitespace();

        match field_it.next() {
            Some(NODE_TAG) => {} // tag matches, continue
            Some(t) => Err(Error::WrongCardTag {
                expect: NODE_TAG.into(),
                actual: t.into(),
            })?,
            None => Err(Error::EmptyLine)?,
        }

        let id_raw = field_it.next().ok_or(Error::MissingValue)?;
        let id = U::from_str_radix(id_raw, 10)?;

        let mut coordinate = [F::zero(); 3];
        for i in 0..3 {
            let Some(c_raw) = field_it.next() else {
                Err(Error::MissingValue)?
            };

            let c = F::from_str_radix(c_raw, 10)?;
            coordinate[i] = c;
        }

        if let Some(v) = field_it.next() {
            weak_error(Error::ExtraneousValue(v.into()))?;
        }

        Ok(Self { id, coordinate })
    }
}
