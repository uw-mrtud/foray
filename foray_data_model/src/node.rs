use derive_more::{Display, Error};
use numpy::{PyReadwriteArrayDyn, ToPyArray, ndarray::ArrayD};
use pyo3::{exceptions::PyTypeError, prelude::*};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub type Dict<K, V> = BTreeMap<K, V>;

pub type Shape = Vec<Option<usize>>;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, PartialOrd)]
pub enum UIParameter {
    NumberField(f64),
    CheckBox(bool),
    Slider(f64, f64, f64),
}
impl UIParameter {
    pub fn default_value(&self) -> PortData {
        match self {
            UIParameter::NumberField(v) => PortData::Float(*v),
            UIParameter::CheckBox(v) => PortData::Boolean(*v),
            UIParameter::Slider(_, _, v) => PortData::Float(*v),
        }
    }
}
impl<'py> FromPyObject<'py> for UIParameter {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        //try to extract String
        match ob.extract::<(String, Dict<String, Bound<'py, PyAny>>)>() {
            Ok((s, o)) => Ok(match s.as_str() {
                "NumberField" => match o.get("default") {
                    Some(o) => match o.extract::<f64>() {
                        Ok(v) => UIParameter::NumberField(v),
                        Err(_) => Err(PyTypeError::new_err("expected a number value"))?,
                    },
                    None => Err(PyTypeError::new_err("expected a 'default' key"))?,
                },
                "CheckBox" => match o.get("default") {
                    Some(o) => match o.extract::<bool>() {
                        Ok(v) => UIParameter::CheckBox(v),
                        Err(_) => Err(PyTypeError::new_err("expected a boolean value"))?,
                    },
                    None => Err(PyTypeError::new_err("expected a 'default' key"))?,
                },
                "Slider" => {
                    let start = match o.get("start") {
                        Some(o) => match o.extract::<f64>() {
                            Ok(v) => v,
                            Err(_) => Err(PyTypeError::new_err("expected a boolean value"))?,
                        },
                        None => Err(PyTypeError::new_err("expected a 'start' key"))?,
                    };
                    let stop = match o.get("stop") {
                        Some(o) => match o.extract::<f64>() {
                            Ok(v) => v,
                            Err(_) => Err(PyTypeError::new_err("expected a boolean value"))?,
                        },
                        None => Err(PyTypeError::new_err("expected a 'stop' key"))?,
                    };
                    let default = match o.get("default") {
                        Some(o) => match o.extract::<f64>() {
                            Ok(v) => v,
                            Err(_) => Err(PyTypeError::new_err("expected a boolean value"))?,
                        },
                        None => Err(PyTypeError::new_err("expected a 'default' key"))?,
                    };

                    UIParameter::Slider(start, stop, default)
                }
                _ => Err(PyTypeError::new_err(format!("Unsupported data type: {s}")))?,
            }),
            Err(_) => Err(PyTypeError::new_err("Unsupported format for parameter"))?,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum PortType {
    Integer,
    Float,
    Boolean,
    String,
    Array(Box<PortType>, Shape),
    Object(Dict<String, PortType>),
}

impl<'py> FromPyObject<'py> for PortType {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        //try to extract String
        match ob.extract::<&str>() {
            Ok(s) => Ok(match s {
                "Integer" => PortType::Integer,
                "Float" => PortType::Float,
                "Boolean" => PortType::Boolean,
                "String" => PortType::String,
                _ => Err(PyTypeError::new_err(format!("Unsupported data type: {s}")))?,
            }),
            Err(_) => match ob.extract::<(PortType, Shape)>() {
                Ok((port_type, shape)) => Ok(PortType::Array(Box::new(port_type), shape)),
                Err(_) => ob.extract::<Dict<String, PortType>>().map(PortType::Object),
            },
        }
    }
}

#[derive(Clone, Serialize, Deserialize, FromPyObject, IntoPyObject, Debug, PartialEq)]
pub enum PortData {
    Integer(i32),
    Float(f64),
    Boolean(bool),
    String(String),
    Array(ForayArray),
    Object(Dict<String, PortData>),
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum ForayArray {
    Integer(ArrayD<i32>),
    Float(ArrayD<f64>),
    Boolean(ArrayD<bool>),
    String(ArrayD<String>),
    // arrays should be flattened into non-nested arrays whenever
    // posible. ArrayD<i32> is much faster than ArrayD<PrimitiveData::Integer(i32)>,
    // but nesting is still possible when needed
    Object(ArrayD<PortData>),
}

impl<'py> FromPyObject<'py> for ForayArray {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        match ob.extract::<PyReadwriteArrayDyn<i32>>() {
            Ok(arr) => Ok(ForayArray::Integer(arr.as_array().to_owned())),
            Err(_) => ob
                .extract::<PyReadwriteArrayDyn<f64>>()
                .map(|arr| ForayArray::Float(arr.as_array().to_owned())),
        }
    }
}

impl<'py> IntoPyObject<'py> for ForayArray {
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

// impl GraphNode<NodeTemplate, PortType, PortData> for NodeTemplate {
//     fn inputs(&self) -> Dict<PortName, PortType> {
//         todo!()
//     }
//
//     fn outputs(&self) -> Dict<PortName, PortType> {
//         todo!()
//     }
//
//     fn compute(
//         self,
//         inputs: Dict<PortName, WireDataContainer<PortData>>,
//     ) -> Result<(Dict<PortName, PortData>, NodeTemplate), NodeError> {
//         todo!()
//     }
// }

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Display, Debug)]
pub enum NodeError {
    Input(String),
    Err,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Error, Display, Debug)]
pub enum PortError {
    Err,
    InvalidPortContent,
    NoPortKey,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Error, Display, Debug, PartialOrd)]
pub enum ParameterError {
    Err,
    InvalidParameterContent,
    NoParameterKey,
}
