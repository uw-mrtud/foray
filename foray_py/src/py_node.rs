use foray_data_model::{
    WireDataContainer,
    node::{Dict, ParameterError, PortData, PortError, PortType, UIParameter},
};

use log::trace;
use pyo3::{
    Bound, PyAny, Python,
    types::{PyAnyMethods, PyModule},
};

use serde::{Deserialize, Serialize};

use crate::err::{PyNodeConfigError, py_err_traceback};

/// Template that will be stored for each available node type
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct PyNodeTemplate {
    pub name: String,
    pub py_path: String,
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
    pub fn new(py_path: String) -> Self {
        trace!("loading node: {py_path:?}");
        let config = load_node(&py_path);
        PyNodeTemplate {
            name: py_path
                .split(".")
                .last()
                .unwrap_or("EMPTY_PY_PATH")
                .to_string(),
            py_path,
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
    let py_inputs: Dict<String, PortData> = populated_inputs
        .into_iter()
        .map(|(k, v)| (k.clone(), v.read().unwrap().clone()))
        .collect();
    py_compute_unlocked(template, py_inputs, populated_parameters)
}

pub fn py_compute_unlocked(
    template: &PyNodeTemplate,
    populated_inputs: Dict<String, PortData>,
    populated_parameters: Dict<String, PortData>,
) -> Result<Dict<String, PortData>, PyNodeConfigError> {
    Python::with_gil(|py| {
        let node_module = PyModule::import(py, &template.py_path)?;
        node_module
            .getattr("compute")?
            .call((populated_inputs, populated_parameters), None)?
            .extract::<Dict<String, PortData>>()
            .map_err(|py_err| PyNodeConfigError::ConfigReturn(py_err.to_string()))
    })
}

fn load_node(py_path: &str) -> Result<PyConfig, PyNodeConfigError> {
    Python::with_gil(|py| {
        let node_module = PyModule::import(py, py_path)?;

        // Reload module to make sure we are up to date
        let import_mod = PyModule::import(py, "importlib")?;
        let _ = import_mod.getattr("reload").unwrap().call1((&node_module,));

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
