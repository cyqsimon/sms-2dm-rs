use std::{fmt, ops::ControlFlow};

use num_traits::Unsigned;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{error::weak_error, DefaultUnsigned, Error};

pub(crate) const NODESTRING_TAG: &str = "NS";
/// Identifies a nodestring.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Nodestring<U = DefaultUnsigned>
where
    U: Unsigned + Copy + fmt::Debug,
    Error: From<U::FromStrRadixErr>,
{
    pub nodes: Vec<U>,
}
impl<U> Nodestring<U>
where
    U: Unsigned + Copy + fmt::Debug,
    Error: From<U::FromStrRadixErr>,
{
    pub(crate) fn new() -> Self {
        Self { nodes: vec![] }
    }

    /// Ingest a nodestring card.
    ///
    /// The returned [`ControlFlow`] indicates whether parsing should continue
    /// into the next line (i.e. `Break` when a tail node is encountered).
    ///
    /// The tail node is denoted by a negative sign prefix.
    pub(crate) fn ingest(&mut self, line: impl AsRef<str>) -> Result<ControlFlow<()>, Error> {
        let line = line.as_ref();
        let mut field_it = line.split_whitespace();

        match field_it.next() {
            Some(NODESTRING_TAG) => {} // tag matches, continue
            Some(t) => Err(Error::WrongCardTag {
                expect: NODESTRING_TAG.into(),
                actual: t.into(),
            })?,
            None => Err(Error::EmptyLine)?,
        }

        while let Some(node_raw) = field_it.next() {
            if let Some(node_n) = node_raw.strip_prefix('-') {
                let node = U::from_str_radix(node_n, 10)?;
                self.nodes.push(node);

                if let Some(val) = field_it.next() {
                    weak_error(Error::ExtraneousValue(val.into()))?;
                }
                return Ok(ControlFlow::Break(()));
            }

            let node = U::from_str_radix(node_raw, 10)?;
            self.nodes.push(node);
        }

        Ok(ControlFlow::Continue(()))
    }
}
