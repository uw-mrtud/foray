use std::{ffi::CString, fs, io, path::PathBuf};

use data_model::node::{Dict, NodeTemplate, ParameterError, PortError, PortType, UIParameter};
use pyo3::{
    Bound, PyAny, PyErr, Python,
    types::{PyAnyMethods, PyModule},
};

// // New type, so we can derive pyclass methods
// #[derive(Clone, Debug)]
// #[pyclass]
// struct PyPortType(PortType);

// // New type, so we can derive pyclass methods
// #[derive(Clone)]
// #[pyclass]
// struct PyUIParameter(UIParameter);

#[derive(Debug)]
pub enum PyNodeError {
    PyErr(PyErr),
    IoErr(io::Error),
}

impl From<PyErr> for PyNodeError {
    fn from(e: PyErr) -> Self {
        Self::PyErr(e)
    }
}
impl From<io::Error> for PyNodeError {
    fn from(e: io::Error) -> Self {
        Self::IoErr(e)
    }
}

pub fn load_node(path: PathBuf) -> Result<NodeTemplate, PyNodeError> {
    let node_src = fs::read_to_string(&path)?;
    parse_node(path, node_src)
}

pub fn parse_node(path: PathBuf, node_src: String) -> Result<NodeTemplate, PyNodeError> {
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
        )?;

        let config_dict = node_module
            .getattr("config")?
            .call0()?
            .extract::<Dict<String, Bound<'_, PyAny>>>()?;
        //
        let (inputs, outputs) = load_ports(&config_dict);
        let parameters = load_parameters(&config_dict);
        Ok(NodeTemplate {
            name: node_name,
            absolute_path: path.clone(),
            inputs,
            outputs,
            relative_path: Default::default(),
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
            .map_err(|_| ParameterError::InvalidParamaterContent),
        None => Ok(Dict::new()),
    }
}
