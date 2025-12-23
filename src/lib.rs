mod element;
mod error;
mod node;
mod nodestring;

use std::{fmt, iter, str::FromStr};

use num_traits::{Float, Unsigned};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub use crate::{element::*, error::Error, node::Node, nodestring::Nodestring};
use crate::{error::weak_error, node::NODE_TAG, nodestring::NODESTRING_TAG};

type DefaultUnsigned = u32;
type DefaultFloat = f64;

pub(crate) const MESH_2D_TAG: &str = "MESH2D";

/// Representation of a `.2dm` file, as defined in https://www.xmswiki.com/wiki/SMS:2D_Mesh_Files_*.2dm.
///
/// You can optionally specify the types of unsigned integers and floats used in this struct using
/// generic types `U` and `F` respectively. The defaults are `U = u32` and `F = f64`.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Mesh2D<U = DefaultUnsigned, F = DefaultFloat>
where
    U: Unsigned + Copy + fmt::Debug,
    F: Float + fmt::Debug,
    Error: From<U::FromStrRadixErr> + From<F::FromStrRadixErr>,
{
    /// Defines number of materials per element.
    ///
    /// Corresponds to the card `NUM_MATERIALS_PER_ELEM`.
    ///
    /// This card is mandatory, but did not exist prior to v11.0.
    pub material_count_per_element: Option<MaterialCountPerElement<U>>,

    /// All nodes defined by the mesh.
    pub nodes: Vec<Node<U, F>>,

    /// All 2-noded linear elements defined by the mesh.
    pub e2ls: Vec<E2L<U>>,
    /// All 3-noded linear elements defined by the mesh.
    pub e3ls: Vec<E3L<U>>,
    /// All 3-noded triangular elements defined by the mesh.
    pub e3ts: Vec<E3T<U>>,
    /// All 6-noded triangular elements defined by the mesh.
    pub e6ts: Vec<E6T<U>>,
    /// All 4-noded quadrilateral elements defined by the mesh.
    pub e4qs: Vec<E4Q<U>>,
    /// All 8-noded quadrilateral elements defined by the mesh.
    pub e8qs: Vec<E8Q<U>>,
    /// All 9-noded quadrilateral elements defined by the mesh.
    pub e9qs: Vec<E9Q<U>>,

    /// All nodestrings defined by the mesh.
    pub nodestring: Vec<Nodestring<U>>,
}
impl<U, F> FromStr for Mesh2D<U, F>
where
    U: Unsigned + Copy + fmt::Debug,
    F: Float + fmt::Debug,
    Error: From<U::FromStrRadixErr> + From<F::FromStrRadixErr>,
{
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut mesh = Mesh2D::new();

        let mut line_it = s.lines();

        // first line must be `MESH2D`
        match line_it.next() {
            Some(MESH_2D_TAG) => {} // continue
            Some(_) | None => weak_error(Error::MissingCard(MESH_2D_TAG.into()))?,
        }

        while let Some(line) = line_it.next() {
            let Some(card_type) = line.split_whitespace().next() else {
                // empty line
                weak_error(Error::EmptyLine)?;
                // skip this line if we don't hard error
                continue;
            };

            macro_rules! parse_push {
                ($field: ident) => {{
                    let val = line.parse()?;
                    mesh.$field.push(val);
                }};
            }
            match card_type {
                MATERIAL_COUNT_PER_ELEMENT_TAG => {
                    // check duplicate
                    if mesh.material_count_per_element.is_some() {
                        weak_error(Error::ExtraneousCard(MATERIAL_COUNT_PER_ELEMENT_TAG.into()))?;
                    }
                    let val = line.parse()?;
                    let _ = mesh.material_count_per_element.insert(val);
                }
                NODE_TAG => parse_push!(nodes),
                E2L_TAG => parse_push!(e2ls),
                E3L_TAG => parse_push!(e3ls),
                E3T_TAG => parse_push!(e3ts),
                E6T_TAG => parse_push!(e6ts),
                E4Q_TAG => parse_push!(e4qs),
                E8Q_TAG => parse_push!(e8qs),
                E9Q_TAG => parse_push!(e9qs),
                NODESTRING_TAG => 'ns_end: {
                    let mut ns = Nodestring::new();
                    for line in iter::once(line).chain(&mut line_it) {
                        if ns.ingest(line)?.is_break() {
                            mesh.nodestring.push(ns);
                            break 'ns_end;
                        }
                    }
                    // no more lines without seeing a tail node
                    // always a hard error because it's too much of a mess otherwise
                    Err(Error::MissingCard(NODESTRING_TAG.into()))?;
                }
                _ => {} // TODO: other cards ignored for now; PRs welcomed
            }
        }

        Ok(mesh)
    }
}
impl<U, F> Mesh2D<U, F>
where
    U: Unsigned + Copy + fmt::Debug,
    F: Float + fmt::Debug,
    Error: From<U::FromStrRadixErr> + From<F::FromStrRadixErr>,
{
    fn new() -> Self {
        Self {
            material_count_per_element: None,
            nodes: vec![],
            e2ls: vec![],
            e3ls: vec![],
            e3ts: vec![],
            e6ts: vec![],
            e4qs: vec![],
            e8qs: vec![],
            e9qs: vec![],
            nodestring: vec![],
        }
    }
}

pub(crate) const MATERIAL_COUNT_PER_ELEMENT_TAG: &str = "NUM_MATERIALS_PER_ELEM";

/// Defines number of materials per element.
///
/// Corresponds to the card ``NUM_MATERIALS_PER_ELEM`.`
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MaterialCountPerElement<U = DefaultUnsigned>(pub U)
where
    U: Unsigned + Copy + fmt::Debug,
    Error: From<U::FromStrRadixErr>;
impl<U> FromStr for MaterialCountPerElement<U>
where
    U: Unsigned + Copy + fmt::Debug,
    Error: From<U::FromStrRadixErr>,
{
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut field_it = s.split_whitespace();

        match field_it.next() {
            Some(MATERIAL_COUNT_PER_ELEMENT_TAG) => {} // tag matches, continue
            Some(t) => Err(Error::WrongCardTag {
                expect: MATERIAL_COUNT_PER_ELEMENT_TAG.into(),
                actual: t.into(),
            })?,
            None => Err(Error::EmptyLine)?,
        }

        let count_raw = field_it.next().ok_or(Error::MissingValue)?;
        let count = U::from_str_radix(count_raw, 10)?;

        if let Some(v) = field_it.next() {
            weak_error(Error::ExtraneousValue(v.into()))?;
        }

        Ok(Self(count))
    }
}

#[cfg(test)]
mod test {
    use std::fs;

    use super::*;

    #[test]
    fn multi_line_nodestring_parse() -> anyhow::Result<()> {
        let input = fs::read_to_string("test-res/multi-line-nodestring.2dm")?;
        let mesh = <Mesh2D>::from_str(&input)?;

        assert_eq!(mesh.nodestring.len(), 1);

        Ok(())
    }

    #[test]
    fn sample_file_2_parse() -> anyhow::Result<()> {
        let input = fs::read_to_string("test-res/sample-2.2dm")?;
        let mesh = <Mesh2D>::from_str(&input)?;

        assert!(mesh.material_count_per_element.is_none());
        assert_eq!(mesh.nodes.len(), 6);
        assert_eq!(mesh.e2ls.len(), 0);
        assert_eq!(mesh.e3ls.len(), 0);
        assert_eq!(mesh.e3ts.len(), 3);
        assert_eq!(mesh.e6ts.len(), 0);
        assert_eq!(mesh.e4qs.len(), 3);
        assert_eq!(mesh.e8qs.len(), 0);
        assert_eq!(mesh.e9qs.len(), 0);
        assert_eq!(mesh.nodestring.len(), 4);

        Ok(())
    }
}
