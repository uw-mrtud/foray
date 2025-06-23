use std::{ffi::CString, fs, path::PathBuf, str::FromStr};

use derive_more::derive::{Debug, Display};
use iced::{
    widget::{column, *},
    Alignment::Center,
};
use iced::{Element, Length::Fill};
use log::trace;
use numpy::{Complex64, PyArrayMethods, ToPyArray};
use pyo3::{
    ffi::c_str,
    types::{PyAnyMethods, PyComplex, PyDict, PyDictMethods, PyModule},
    Bound, FromPyObject, IntoPyObject, PyAny, PyErr, PyObject, PyResult, Python,
};
use relative_path::RelativePathBuf;
use serde::{Deserialize, Serialize};
use strum::VariantNames;

use crate::{
    app::Message,
    interface::node_config::{NodeUIParameters, NodeUIWidget},
    StableMap,
};
use crate::{
    gui_node::PortDataReference,
    nodes::{
        port::{PortData, PortType},
        status::NodeError,
    },
};

#[derive(Clone, Debug, Display, Serialize, Deserialize, PartialEq, PartialOrd)]
#[display("{}", self.name)]
pub struct PyNode {
    pub name: String,
    #[serde(skip)]
    pub absolute_path: PathBuf,
    pub relative_path: RelativePathBuf,
    pub ports: Result<PortDef, NodeError>,
    pub parameters: Result<NodeUIParameters, NodeError>,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq, FromPyObject, PartialOrd)]
pub struct PortDef {
    pub inputs: StableMap<String, PortType>,
    pub outputs: StableMap<String, PortType>,
}

impl<'py> FromPyObject<'py> for PortType {
    fn extract_bound(ob: &pyo3::Bound<'py, pyo3::PyAny>) -> pyo3::PyResult<Self> {
        if let Ok(s) = ob.extract::<String>() {
            match PortType::from_str(&s) {
                Ok(pt) => Ok(pt),
                Err(_) => PyResult::Err(PyErr::from_value(ob.clone())),
            }
        } else {
            Ok(PortType::Object(
                ob.extract::<StableMap<String, PortType>>()?
                    .into_iter()
                    .collect(),
            ))
        }
    }
}

impl PortData {
    pub fn to_py(&self, py: Python) -> PyObject {
        match self {
            PortData::Integer(val) => val.into_pyobject(py).expect("valid python integer").into(),
            PortData::Real(val) => val.into_pyobject(py).expect("valid python float").into(),
            PortData::Complex(val) => PyComplex::from_doubles(py, val.re, val.im).into(),
            PortData::ArrayReal(array_base) => array_base.to_pyarray(py).into_any().into(),
            PortData::ArrayInteger(array_base) => array_base.to_pyarray(py).into_any().into(),
            PortData::ArrayComplex(array_base) => array_base.to_pyarray(py).into_any().into(),
            PortData::Dynamic(array_base) => array_base.to_pyarray(py).into_any().into(),
            PortData::Object(obj) => {
                let dict = PyDict::new(py);
                obj.iter().for_each(|(k, v)| {
                    let _ = dict.set_item(k, v.to_py(py));
                });
                dict.into()
            }
        }
    }
}

impl PyNode {
    pub fn new(absolute_path: PathBuf, relative_path: RelativePathBuf) -> Self {
        //// Get necessary info
        let node_name = relative_path.file_stem().expect("node exists").to_string();

        let node_src = match fs::read_to_string(&absolute_path)
            .map_err(|e| NodeError::FileSys(e.to_string()))
        {
            Ok(src) => src,
            Err(_) => {
                let py_node = PyNode {
                    name: node_name.to_string(),
                    absolute_path,
                    relative_path,
                    ports: Err(NodeError::FileSys("Could not find src file".into())),
                    parameters: Err(NodeError::FileSys("Could not find src file".into())),
                };
                log::error!("Failed to load node {node_name} {py_node:?}");
                return py_node;
            }
        };
        //// Call into python
        Python::with_gil(|py| {
            trace!("Reading node config '{node_name}'");

            let read_src = || -> Result<Bound<PyModule>, NodeError> {
                PyModule::from_code(
                    py,
                    CString::new(node_src)
                        .map_err(|e| {
                            NodeError::Syntax(format!("Error parsing node '{node_name}'\n{e}"))
                        })?
                        .as_c_str(),
                    CString::new(format!("{node_name}.py"))
                        .map_err(|e| {
                            NodeError::Syntax(format!("Error with node name {node_name}\n{e}"))
                        })?
                        .as_c_str(),
                    CString::new(node_name.to_string())
                        .map_err(|e| {
                            NodeError::Syntax(format!("Error with node name {node_name}\n{e}"))
                        })?
                        .as_c_str(),
                )
                .map_err(|e| NodeError::Syntax(format!("Error in node '{node_name}' \n{e}")))
            };

            //TODO Clean up error handling
            match read_src() {
                Ok(module) => {
                    let config: Result<Bound<PyAny>, NodeError> = module
                        .getattr("config")
                        .map_err(|_e| {
                            NodeError::Config(format!(
                                "Could not access 'config' function for {node_name}"
                            ))
                        })
                        .and_then(|module| {
                            module.call0().map_err(|e| {
                                NodeError::Config(format!(
                                    "could not determine '{node_name}' config: {}",
                                    e
                                ))
                            })
                        });

                    let ports = config.clone().and_then(|c| {
                        c.extract::<PortDef>()
                            .map_err(|e| NodeError::Output(e.to_string()))
                    });

                    let parameters = config
                            .and_then(|c| {
                                c.getattr("parameters").map_err(|_e| {
                                    NodeError::Config(
                                        "'parameters' attribute not found, does it exist for the node?"
                                            .to_string(),
                                    )
                                })
                            })
                            .and_then(|out_py| {
                                out_py
                                    .extract::<StableMap<String, String>>()
                                    .map_err(|e| {
                                        NodeError::Config(format!(
                                        "Failed to interperet  {node_name}'s `config`: {e}, {out_py}"
                                    ))
                                    })?
                                    .into_iter()
                                    .map(|(k, v)| {
                                        NodeUIWidget::from_str(&v)
                                            .map(|v| (k, v))
                                            .map_err(|_e| NodeError::Other)
                                    })
                                    .collect()
                            });

                    PyNode {
                        name: node_name.to_string(),
                        absolute_path,
                        relative_path,
                        ports,
                        parameters,
                    }
                }
                Err(e) => PyNode {
                    name: node_name.to_string(),
                    absolute_path,
                    relative_path,
                    ports: Err(e.clone()),
                    parameters: Err(e),
                },
            }
        })
    }

    pub fn compute(
        &self,
        inputs: StableMap<String, PortDataReference>,
    ) -> Result<StableMap<String, PortData>, NodeError> {
        match &self.ports {
            Ok(_ports) => {
                // Convert inputs to python arrays/objects
                Python::with_gil(|py| {
                    let py_inputs = inputs
                        .into_iter()
                        .map(|(k, v)| (k.clone(), v.to_py(py)))
                        .collect();
                    //TODO: refactor to not pass this data as params
                    self.gpipy_compute(
                        &self.absolute_path,
                        &py_inputs,
                        &self.parameters.clone().unwrap_or_default(),
                        py,
                    )
                    .map_err(|err| NodeError::Runtime(err.to_string()))
                })
            }
            // If the ports are not valid, don't bother running. Just surface the error
            Err(e) => Err(e.clone()),
        }
    }

    #[allow(clippy::complexity)]
    /// Run a python node's compute function
    fn gpipy_compute<'py>(
        &self,
        node_path: &PathBuf,
        inputs: &StableMap<String, PyObject>,
        parameters: &NodeUIParameters,
        py: Python<'py>,
    ) -> Result<StableMap<String, PortData>, NodeError> {
        if let Some(node_name) = node_path.file_stem() {
            //TODO: use self parameters, instead of taking unecessary inputs
            //PERF: cache this in the PyNode
            let node_src = fs::read_to_string(node_path.clone())
                .map_err(|e| NodeError::FileSys(e.to_string()))?;
            trace!("Running '{:?}' compute:\n{node_src}", node_path.file_stem());
            //PERF: test if caching this is a big performance win
            //This would be more of a pain to cache becaues of the associated python lifetime, but could
            //potentially be worth it
            //Update: It may be possible to package the py type without a lifetime? pyo3 docs

            let node_module = PyModule::from_code(
                py,
                CString::new(node_src)
                    .map_err(|e| NodeError::FileSys(e.to_string()))?
                    .as_c_str(),
                &CString::new(format!("{:?}.py", node_name))
                    .expect("Node names should not contain invalid characters"),
                c_str!("gpi_node"),
            )
            .map_err(|e| NodeError::Syntax(e.to_string()))?;

            //// COMPUTE
            let node_output = node_module
                .getattr("compute")
                .map_err(|_| {
                    NodeError::MissingCompute("Could not find compute function".to_string())
                })?
                .call(
                    (
                        inputs.iter().collect::<StableMap<_, _>>(),
                        parameters
                            .iter()
                            .map(|(k, v)| (k, v.to_string()))
                            .collect::<StableMap<_, _>>(),
                    ),
                    None,
                )
                .map_err(|e| NodeError::Runtime(format!("Python Error:\n{e}")))?;

            node_output
                .extract::<StableMap<String, PyObject>>()
                .map_err(|e| {
                    NodeError::Output(format!("Unable to understand python return value:\n{e}"))
                })?
                .into_iter()
                .map(|(k, v)| {
                    Ok((
                        k.clone(),
                        PyNode::extract_py_data(
                            &self
                                .ports
                                .as_ref()
                                .expect("ports must be valid to compute")
                                .outputs[&k],
                            &v,
                            py,
                        )
                        .map_err(|e| {
                            NodeError::Output(format!(
                                "Unable to understand port in python return value:\n{e}"
                            ))
                        })?,
                    ))
                })
                .collect::<Result<StableMap<String, PortData>, NodeError>>()
        } else {
            panic!("Node not found {:?}", node_path)
        }
    }

    pub fn extract_py_data(
        port_type: &PortType,
        py_object: &PyObject,
        py: Python,
    ) -> Result<PortData, NodeError> {
        // unsure how to make the repetion bellow more generic, while still
        // automatically converting to the PortType
        unsafe {
            Ok(match port_type {
                PortType::Integer => PortData::Integer(
                    py_object
                        .bind(py)
                        .extract()
                        .map_err(|_e| output_error(port_type, py_object))?,
                ),
                PortType::Real => PortData::Real(
                    py_object
                        .bind(py)
                        .extract()
                        .map_err(|_e| output_error(port_type, py_object))?,
                ),
                PortType::Complex => PortData::Complex(
                    py_object
                        .bind(py)
                        .extract::<(f64, f64)>()
                        .map(|(r, i)| Complex64::new(r, i))
                        .map_err(|_e| output_error(port_type, py_object))?,
                ),
                PortType::ArrayReal => PortData::ArrayReal(
                    py_object
                        .bind(py)
                        .downcast()
                        .map_err(|_e| output_error(port_type, py_object))?
                        .as_array()
                        .to_owned(),
                ),
                PortType::ArrayInteger => PortData::ArrayInteger(
                    py_object
                        .bind(py)
                        .downcast()
                        .map_err(|_e| output_error(port_type, py_object))?
                        .as_array()
                        .to_owned(),
                ),
                PortType::ArrayComplex => PortData::ArrayComplex(
                    py_object
                        .bind(py)
                        .downcast()
                        .map_err(|_e| output_error(port_type, py_object))?
                        .as_array()
                        .to_owned(),
                ),
                PortType::Dynamic => PortData::Dynamic(
                    py_object
                        .bind(py)
                        .downcast()
                        .map_err(|_e| output_error(port_type, py_object))?
                        .as_array()
                        .to_owned(),
                ),
                PortType::Object(types) => {
                    let dict: &Bound<PyDict> = py_object
                        .bind(py)
                        .downcast()
                        .map_err(|_e| output_error(port_type, py_object))?;
                    let rust_dict: StableMap<String, PortData> = dict
                        .iter()
                        .map(|(k, v)| {
                            let key = k.extract::<String>().map_err(|e| {
                                NodeError::Output(format!("Failed to read output port name {e}"))
                            });
                            key.map(|k| {
                                (
                                    k.clone(),
                                    Self::extract_py_data(&types[&k], &v.into(), py).unwrap(),
                                )
                            })
                        })
                        .collect::<Result<_, _>>()?;
                    PortData::Object(rust_dict)
                }
            })
        }
    }

    pub(crate) fn config_view(
        &self,
        id: u32,
        _input_data: StableMap<String, std::sync::Arc<std::sync::RwLock<PortData>>>,
    ) -> Option<Element<'_, Message>> {
        if let Ok(parameters) = &self.parameters {
            Some(
                column(parameters.iter().map(|(name, widget_type)| {
                    let message = move |widget_value| {
                        Message::UpdateNodeParameter(id, name.to_string(), widget_value)
                    };
                    row![text(name), widget_type.view(message)]
                        .spacing(8.0)
                        .align_y(Center)
                        .width(Fill)
                        .into()
                }))
                .spacing(8.)
                .width(Fill)
                .into(),
            )
        } else {
            Some(text("").into())
        }
    }
}

/// Port to receive port def from python
#[derive(Clone, FromPyObject, Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct PyFacingNodeDef {
    pub inputs: StableMap<String, PortType>,
    pub outputs: StableMap<String, PortType>,
    pub parameters: StableMap<String, String>,
}

impl TryFrom<PyFacingNodeDef> for NodeUIParameters {
    type Error = NodeError;
    fn try_from(value: PyFacingNodeDef) -> Result<Self, Self::Error> {
        value
            .clone()
            .parameters
            .into_iter()
            .map(|(key, value)| NodeUIWidget::from_str(&value).map(|v| (key, v)))
            .collect::<Result<StableMap<_, _>, _>>()
            .map_err(|e| {
                NodeError::Output(format!(
                    "{:#?}\nExpected one of {:#?}, found {:#?}",
                    e,
                    NodeUIWidget::VARIANTS,
                    value.parameters,
                ))
            })
    }
}

fn output_error(port_type: &PortType, py_object: &PyObject) -> NodeError {
    NodeError::Output(format!(
        "Received unexpected output from node. Expected one of {port_type:#?}, found {py_object:#?}"
    ))
}
