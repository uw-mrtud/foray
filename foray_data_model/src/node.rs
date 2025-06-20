use derive_more::{Display, Error};
use numpy::{PyReadwriteArrayDyn, ToPyArray, ndarray::ArrayD};
use pyo3::prelude::*;
use relative_path::RelativePathBuf;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::PathBuf, time::Instant};

use crate::node_spec::GraphNode;
pub type Dict<K, V> = BTreeMap<K, V>;

pub type Shape = Vec<Option<usize>>;

#[pyclass]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum UIParameter {
    NumberField,
    CheckBox,
    Slider,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Copy)]
pub enum PrimitiveType {
    Integer,
    Float,
    Boolean,
    String,
}

impl<'py> FromPyObject<'py> for PrimitiveType {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let string = ob.extract::<String>()?;
        Ok(match string.as_str() {
            "Integer" => PrimitiveType::Integer,
            "Float" => PrimitiveType::Float,
            "Boolean" => PrimitiveType::Boolean,
            "String" => PrimitiveType::String,
            _ => todo!(),
        })
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum PortType {
    Primitive(PrimitiveType),
    Array(Box<PortType>, Shape),
    Object(Dict<String, PortType>),
}

impl<'py> FromPyObject<'py> for PortType {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        match ob.extract::<PrimitiveType>() {
            Ok(p) => Ok(PortType::Primitive(p)),
            Err(_) => match ob.extract::<(PortType, Shape)>() {
                Ok((port_type, shape)) => Ok(PortType::Array(Box::new(port_type), shape)),
                Err(_) => ob.extract::<Dict<String, PortType>>().map(PortType::Object),
            },
        }
    }
}

// #[pyclass]
#[derive(Serialize, Deserialize, Clone, FromPyObject, IntoPyObject, Debug)]
pub enum PrimitiveData {
    Integer(i32),
    Float(f64),
    Boolean(bool),
    String(String),
}

#[derive(Clone, Serialize, Deserialize, FromPyObject, IntoPyObject, Debug)]
pub enum PortData {
    Primitve(PrimitiveData),
    Array(PrimitiveArray),
    Object(Dict<String, PortData>),
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum PrimitiveArray {
    Integer(ArrayD<i32>),
    Float(ArrayD<f64>),
    Boolean(ArrayD<bool>),
    String(ArrayD<String>),
    // arrays should be flattened into non-nested arrays whenever
    // posible. ArrayD<i32> is much faster than ArrayD<PrimitiveData::Integer(i32)>,
    // but nesting is still possible when needed
    Object(ArrayD<PortData>),
}

impl<'py> FromPyObject<'py> for PrimitiveArray {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        match ob.extract::<PyReadwriteArrayDyn<i32>>() {
            Ok(arr) => Ok(PrimitiveArray::Integer(arr.as_array().to_owned())),
            Err(_) => ob
                .extract::<PyReadwriteArrayDyn<f64>>()
                .map(|arr| PrimitiveArray::Float(arr.as_array().to_owned())),
        }
    }
}

impl<'py> IntoPyObject<'py> for PrimitiveArray {
    type Target = PyAny;

    type Output = Bound<'py, Self::Target>;

    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(match self {
            Self::Integer(array_base) => array_base.to_pyarray(py).into_any(),
            Self::Float(array_base) => array_base.to_pyarray(py).into_any(),
            Self::Boolean(array_base) => array_base.to_pyarray(py).into_any(),
            _ => todo!(), // Self::String(array_base) => array_base
                          //     .iter()
                          //     .map(|e| e.into_pyobject(py))
                          //     .collect()
                          //     .to_pyarray(py),
                          // Self::Object(array_base) => array_base
                          //     .iter()
                          //     .map(|e| e.into_pyobject(py))
                          //     .collect()
                          //     .to_pyarray(py)
                          //     .into_any()
                          //     .into(),
        })
    }
}

/// Template that will be stored for each available node type
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct NodeTemplate {
    pub name: String,
    #[serde(skip)]
    pub absolute_path: PathBuf,
    pub relative_path: RelativePathBuf,
    pub inputs: Result<Dict<String, PortType>, PortError>,
    pub outputs: Result<Dict<String, PortType>, PortError>,
    pub parameters: Result<Dict<String, UIParameter>, ParameterError>,
}

impl GraphNode<NodeTemplate, PortType, PortData> for NodeTemplate {
    fn inputs(&self) -> Dict<crate::node_spec::PortName, PortType> {
        todo!()
    }

    fn outputs(&self) -> Dict<crate::node_spec::PortName, PortType> {
        todo!()
    }

    fn compute(
        self,
        inputs: Dict<crate::node_spec::PortName, crate::node_spec::WireDataContainer<PortData>>,
    ) -> Result<(Dict<crate::node_spec::PortName, PortData>, NodeTemplate), NodeError> {
        todo!()
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Error, Display, Debug)]
pub enum NodeError {
    Err,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Error, Display, Debug)]
pub enum PortError {
    Err,
    InvalidPortContent,
    NoPortKey,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Error, Display, Debug)]
pub enum ParameterError {
    Err,
    InvalidParamaterContent,
    NoParameterKey,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum NodeStatus {
    #[default]
    Idle,
    Running {
        start: Instant,
    },
}

#[derive(Serialize, Deserialize)]
pub struct NodeInstance {
    // TODO: should this be just an identifier, and keep NodeTemplates
    // in one spot, refering to them when necessary?
    pub node_template: NodeTemplate,
    pub parameters_values: Dict<String, PortData>,
    #[serde(skip)]
    // If there are errors for any of NodeDefinition fields, the field will be empty,
    // The error will be noted in NodeStatus
    pub node_status: NodeStatus,
}
