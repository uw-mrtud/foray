use std::{ffi::CString, fs, path::PathBuf};

use foray_data_model::{
    WireDataContainer,
    node::{Dict, ParameterError, PortData, PortError, PortType, UIParameter},
};

use log::trace;
use pyo3::{
    Bound, PyAny, Python,
    types::{PyAnyMethods, PyModule},
};

use relative_path::RelativePathBuf;
use serde::{Deserialize, Serialize};

use crate::err::{PyNodeConfigError, py_err_traceback};

/// Template that will be stored for each available node type
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct PyNodeTemplate {
    pub name: String,
    #[serde(skip)]
    pub absolute_path: PathBuf,
    pub relative_path: RelativePathBuf,
    pub config: Result<PyConfig, PyNodeConfigError>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct PyConfig {
    pub inputs: Result<Dict<String, PortType>, PortError>,
    pub outputs: Result<Dict<String, PortType>, PortError>,
    pub parameters: Result<Dict<String, UIParameter>, ParameterError>,
}
impl Default for PyConfig {
    fn default() -> Self {
        PyConfig {
            inputs: Ok(Default::default()),
            outputs: Ok(Default::default()),
            parameters: Ok(Default::default()),
        }
    }
}

impl PyNodeTemplate {
    pub fn new(path: PathBuf, relative_path: RelativePathBuf) -> Self {
        trace!("loading node: {path:?}");
        let config = load_node(path.clone());
        PyNodeTemplate {
            name: path.file_stem().unwrap().to_string_lossy().into(),
            absolute_path: path,
            relative_path,
            config,
        }
    }
    pub fn inputs(&self) -> Result<Dict<String, PortType>, PyNodeConfigError> {
        match &self.config {
            Ok(c) => c.inputs.clone().map_err(PyNodeConfigError::Port),
            Err(e) => Err(e.clone()),
        }
    }
    pub fn outputs(&self) -> Result<Dict<String, PortType>, PyNodeConfigError> {
        match &self.config {
            Ok(c) => c.outputs.clone().map_err(PyNodeConfigError::Port),
            Err(e) => Err(e.clone()),
        }
    }
    pub fn parameters(&self) -> Result<Dict<String, UIParameter>, PyNodeConfigError> {
        match &self.config {
            Ok(c) => c.parameters.clone().map_err(PyNodeConfigError::Parameter),
            Err(e) => Err(e.clone()),
        }
    }
    pub fn errors(&self) -> Vec<PyNodeConfigError> {
        match &self.config {
            Ok(c) => c.errors(),
            Err(e) => vec![e.clone()],
        }
    }
}
impl PyConfig {
    pub fn errors(&self) -> Vec<PyNodeConfigError> {
        let mut errors = Vec::new();

        if let Err(e) = &self.inputs {
            errors.push(PyNodeConfigError::Port(e.clone()));
        }
        if let Err(e) = &self.outputs {
            errors.push(PyNodeConfigError::Port(e.clone()));
        }

        if let Err(e) = &self.parameters {
            errors.push(PyNodeConfigError::Parameter(e.clone()));
        }
        errors
    }
}

pub fn py_compute(
    template: &PyNodeTemplate,
    populated_inputs: Dict<String, WireDataContainer<PortData>>,
    populated_parameters: Dict<String, PortData>,
) -> Result<Dict<String, PortData>, PyNodeConfigError> {
    let path = &template.absolute_path;
    let node_src = fs::read_to_string(path)?;
    let py_inputs: Dict<String, PortData> = populated_inputs
        .into_iter()
        .map(|(k, v)| (k.clone(), v.read().unwrap().clone()))
        .collect();
    py_compute_unlocked(template, node_src, py_inputs, populated_parameters)
}

pub fn py_compute_unlocked(
    template: &PyNodeTemplate,
    node_src: String,
    populated_inputs: Dict<String, PortData>,
    populated_parameters: Dict<String, PortData>,
) -> Result<Dict<String, PortData>, PyNodeConfigError> {
    let path = &template.absolute_path;
    let node_name = path
        .file_stem()
        .expect("node path should be a file")
        .to_string_lossy()
        .to_string();
    Python::with_gil(|py| {
        let node_module = PyModule::from_code(
            py,
            &CString::new(node_src).expect("node src should not have malformed text"),
            &CString::new(format!("{node_name}.py"))
                .expect("node name should not have malformed text"),
            &CString::new(node_name.clone()).expect("node name should not have malformed text"),
        )
        .map_err(|py_err| PyNodeConfigError::Runtime(py_err_traceback(py_err)))?;

        node_module
            .getattr("compute")?
            .call((populated_inputs, populated_parameters), None)?
            .extract::<Dict<String, PortData>>()
            .map_err(|py_err| PyNodeConfigError::ConfigReturn(py_err.to_string()))
    })
}

fn load_node(path: PathBuf) -> Result<PyConfig, PyNodeConfigError> {
    let node_src = fs::read_to_string(&path)?;
    parse_node(path, node_src)
}

pub fn parse_node(path: PathBuf, node_src: String) -> Result<PyConfig, PyNodeConfigError> {
    let node_name = path
        .file_stem()
        .expect("node path should be a file")
        .to_string_lossy()
        .to_string();

    Python::with_gil(|py| {
        let node_module = PyModule::from_code(
            py,
            &CString::new(node_src).expect("node src should not have malformed text"),
            &CString::new(format!("{node_name}.py"))
                .expect("node name should not have malformed text"),
            &CString::new(node_name.clone()).expect("node name should not have malformed text"),
        )
        .map_err(|py_err| PyNodeConfigError::Runtime(py_err_traceback(py_err)))?;

        let config_dict = node_module
            .getattr("config")
            .map_err(|_| PyNodeConfigError::NoConfig)?
            .call0()
            .map_err(|py_err| PyNodeConfigError::Runtime(py_err_traceback(py_err)))?
            .extract::<Dict<String, Bound<'_, PyAny>>>()
            .map_err(|py_err| PyNodeConfigError::Runtime(py_err_traceback(py_err)))?;

        let (inputs, outputs) = load_ports(&config_dict);
        let parameters = load_parameters(&config_dict);
        Ok(PyConfig {
            inputs,
            outputs,
            parameters,
        })
    })
}

type PortResult = Result<Dict<String, PortType>, PortError>;
fn load_ports(config_dict: &Dict<String, Bound<'_, PyAny>>) -> (PortResult, PortResult) {
    // Extracts input or ouptut ports
    let extract_port = |ports: Option<&Bound<'_, PyAny>>| match ports {
        Some(dict) => dict
            .extract::<Dict<String, PortType>>()
            .map_err(|_| PortError::InvalidPortContent),
        None => Ok(Dict::new()),
    };

    (
        extract_port(config_dict.get("inputs")),
        extract_port(config_dict.get("outputs")),
    )
}

fn load_parameters(
    config_dict: &Dict<String, Bound<'_, PyAny>>,
) -> Result<Dict<String, UIParameter>, ParameterError> {
    match config_dict.get("parameters") {
        Some(dict) => dict
            .extract::<Dict<String, UIParameter>>()
            .map_err(|_| ParameterError::InvalidParameterContent),
        None => Ok(Dict::new()),
    }
}
