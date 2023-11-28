mod element;
mod error;
mod node;
mod nodestring;

use std::{fmt, str::FromStr};

use num_traits::{Float, Unsigned};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub use crate::{element::*, error::Error, node::Node, nodestring::Nodestring};

type DefaultUnsigned = u32;
type DefaultFloat = f64;

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
        let Some("MESH2D") = line_it.next() else {
            Err(Error::MissingCard("MESH2D".into()))?
        };

        while let Some(line) = line_it.next() {
            let Some(card_type) = line.split_whitespace().next() else {
                // empty line
                Err(Error::EmptyLine)?
            };

            macro_rules! parse_push {
                ($field: ident) => {{
                    let val = line.parse()?;
                    mesh.$field.push(val);
                }};
            }
            match card_type {
                "NUM_MATERIALS_PER_ELEM" => {
                    let val = line.parse()?;
                    if let Some(_) = mesh.material_count_per_element {
                        Err(Error::ExtraneousCard("NUM_MATERIALS_PER_ELEM".into()))?;
                    }
                    let _ = mesh.material_count_per_element.insert(val);
                }
                "ND" => parse_push!(nodes),
                "E2L" => parse_push!(e2ls),
                "E3L" => parse_push!(e3ls),
                "E3T" => parse_push!(e3ts),
                "E6T" => parse_push!(e6ts),
                "E4Q" => parse_push!(e4qs),
                "E8Q" => parse_push!(e8qs),
                "E9Q" => parse_push!(e9qs),
                "NS" => {
                    let mut ns = Nodestring::new();
                    // ingest the first line
                    let multi_line = ns.ingest(line)?;
                    // ingest potential subsequent lines
                    if multi_line {
                        loop {
                            let Some(line) = line_it.next() else {
                                // no more lines without seeing a tail node
                                Err(Error::MissingCard("NS".into()))?
                            };
                            let keep_going = ns.ingest(line)?;
                            if !keep_going {
                                break;
                            }
                        }
                    }
                    mesh.nodestring.push(ns);
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

        let Some("NUM_MATERIALS_PER_ELEM") = field_it.next() else {
            panic!(r#"Material count per element tag should be "NUM_MATERIALS_PER_ELEM""#);
        };

        let count_raw = field_it.next().ok_or(Error::MissingValue)?;
        let count = U::from_str_radix(count_raw, 10)?;

        if let Some(v) = field_it.next() {
            Err(Error::ExtraneousValue(v.into()))?;
        }

        Ok(Self(count))
    }
}
