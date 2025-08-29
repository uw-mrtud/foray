use derive_more::{Display, Error};
use numpy::{Complex64, PyReadwriteArrayDyn, ToPyArray, ndarray::ArrayD};
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
        // TODO: make some helper functions for this
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
    Complex,
    Boolean,
    String,
    Array(Box<PortType>, Shape),
    Object(Dict<String, PortType>),
}

impl<'py> FromPyObject<'py> for PortType {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        match ob.extract::<&str>() {
            Ok(s) => Ok(match s {
                "Integer" => PortType::Integer,
                "Float" => PortType::Float,
                "Complex" => PortType::Complex,
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

#[derive(Clone, Serialize, IntoPyObject, FromPyObject, Deserialize, Debug, PartialEq)]
pub enum PortData {
    Integer(i32),
    Float(f64),
    Complex((f64, f64)),
    Boolean(bool),
    String(String),
    Array(ForayArray),
    Object(Dict<String, PortData>),
}

impl From<&PortData> for PortType {
    fn from(value: &PortData) -> Self {
        match value {
            PortData::Integer(_) => PortType::Integer,
            PortData::Float(_) => PortType::Float,
            PortData::Complex(_) => PortType::Complex,
            PortData::Boolean(_) => PortType::Boolean,
            PortData::String(_) => PortType::String,
            PortData::Array(foray_array) => foray_array.into(),
            PortData::Object(obj) => PortType::Object(
                obj.iter()
                    .map(|(key, port_data)| (key.clone(), port_data.into()))
                    .collect(),
            ),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum ForayArray {
    Integer(ArrayD<i32>),
    Float(ArrayD<f64>),
    Complex(ArrayD<Complex64>),
    Boolean(ArrayD<bool>),
    String(ArrayD<String>),
    // Arrays should be flattened into non-nested arrays whenever
    // posible. ArrayD<i32> is much faster than ArrayD<PrimitiveData::Integer(i32)>,
    // but nesting is still possible when needed
    Object(ArrayD<PortData>),
}
impl From<&ForayArray> for PortType {
    fn from(value: &ForayArray) -> Self {
        fn array_to_shape<A>(a: &ArrayD<A>) -> Shape {
            a.shape().iter().map(|l| Some(*l)).collect()
        }

        match value {
            ForayArray::Integer(array) => {
                PortType::Array(Box::new(PortType::Integer), array_to_shape(array))
            }
            ForayArray::Float(array) => {
                PortType::Array(Box::new(PortType::Float), array_to_shape(array))
            }
            ForayArray::Complex(array) => {
                PortType::Array(Box::new(PortType::Complex), array_to_shape(array))
            }
            ForayArray::Boolean(array) => {
                PortType::Array(Box::new(PortType::Boolean), array_to_shape(array))
            }
            ForayArray::String(array) => {
                PortType::Array(Box::new(PortType::String), array_to_shape(array))
            }
            ForayArray::Object(array) => PortType::Array(
                Box::new(
                    array
                        .first()
                        .map(|pt| pt.into())
                        .unwrap_or(PortType::Object(Default::default())),
                ),
                array_to_shape(array),
            ),
        }
    }
}

impl<'py> FromPyObject<'py> for ForayArray {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let un_extracted = match ob.extract::<PyReadwriteArrayDyn<i32>>() {
            Ok(arr) => return Ok(ForayArray::Integer(arr.as_array().to_owned())),
            Err(_) => ob,
        };
        let un_extracted = match un_extracted.extract::<PyReadwriteArrayDyn<f64>>() {
            Ok(arr) => return Ok(ForayArray::Float(arr.as_array().to_owned())),
            Err(_) => ob,
        };
        let un_extracted = match un_extracted.extract::<PyReadwriteArrayDyn<Complex64>>() {
            Ok(arr) => return Ok(ForayArray::Complex(arr.as_array().to_owned())),
            Err(_) => ob,
        };
        let _un_extracted = match un_extracted.extract::<PyReadwriteArrayDyn<bool>>() {
            Ok(arr) => return Ok(ForayArray::Boolean(arr.as_array().to_owned())),
            Err(_) => ob,
        };
        panic!("Arrays of Strings or Objects, are not supported yet")
        //TODO: Support arrays of strings and objects
        //let un_extracted = match un_extracted.extract::<PyReadwriteArrayDyn<String>>() {
        //    Ok(arr) => return Ok(ForayArray::String(arr.as_array().to_owned())),
        //    Err(_) => ob,
        //};
        //let un_extracted = match un_extracted.extract::<PyReadwriteArrayDyn<PortData>>() {
        //    Ok(arr) => return Ok(ForayArray::Object(arr.as_array().to_owned())),
        //    Err(pyerr) => return Err(pyerr),
        //};
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
            Self::Complex(array_base) => array_base.to_pyarray(py).into_any(),
            Self::Boolean(array_base) => array_base.to_pyarray(py).into_any(),
            _ => panic!("Arrays of Strings or Objects, are not supported yet"),
            //TODO: Support arrays of strings and objects
            //Self::String(array_base) => array_base.to_pyarray(py).into_any(),
            //Self::Object(array_base) => array_base.to_pyarray(py).into_any(),
        })
    }
}

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
