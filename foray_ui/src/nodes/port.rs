use derive_more::derive::Display;
use itertools::Itertools;
use ndarray::{ArrayD, ArrayView, AsArray};
use numpy::Complex64;
use serde::{Deserialize, Serialize};
use strum::{EnumDiscriminants, EnumString, VariantNames};

use crate::StableMap;

#[derive(
    Clone, Display, Debug, EnumString, VariantNames, PartialEq, Serialize, Deserialize, PartialOrd,
)]
pub enum PortType {
    Integer,
    Real,
    Complex,
    ArrayInteger,
    ArrayReal,
    ArrayComplex,
    Dynamic,
    #[display("{_0:?}")]
    Object(StableMap<String, PortType>),
}

impl Default for PortType {
    fn default() -> Self {
        Self::Object(StableMap::default())
    }
}

//PERF: consider ArcArray
#[derive(Clone, Debug, EnumDiscriminants)]
pub enum PortData {
    Integer(i64),
    Real(f64),
    Complex(Complex64),
    ArrayInteger(ArrayD<i64>),
    ArrayReal(ArrayD<f64>),
    ArrayComplex(ArrayD<Complex64>),
    Dynamic(ArrayD<f64>),
    Object(StableMap<String, PortData>),
}

fn write_nd_array<'a, A, T, D>(data: T) -> String
where
    T: AsArray<'a, A, D>,
    D: ndarray::Dimension,
    A: 'a + derive_more::Debug,
{
    let data: ArrayView<'a, A, D> = data.into();

    format!(
        "dim: {:?} {:.2?}",
        data.dim(),
        data.iter().take(10).collect::<Vec<_>>()
    )
}

impl std::fmt::Display for PortData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                PortData::Integer(val) => val.to_string(),
                PortData::Real(val) => val.to_string(),
                PortData::Complex(val) => val.to_string(),
                PortData::ArrayInteger(array_base) => write_nd_array(array_base),
                PortData::ArrayReal(array_base) => write_nd_array(array_base),
                PortData::ArrayComplex(array_base) => write_nd_array(array_base),
                PortData::Dynamic(array_base) => write_nd_array(array_base),
                PortData::Object(index_map) => index_map
                    .iter()
                    .map(|(k, v)| format!("{k}: {v}"))
                    .join("\n"),
            }
        )
    }
}
