use std::{fmt::Display, io};

use foray_data_model::node::{ParameterError, PortError};
use pyo3::{PyErr, Python, types::PyTracebackMethods};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Serialize, Deserialize)]
pub enum PyNodeConfigError {
    Runtime(RuntimeErr),
    NoConfig,
    ConfigReturn(String),
    Port(PortError),
    Parameter(ParameterError),
    Io(String),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Serialize, Deserialize)]
pub struct RuntimeErr {
    pub error: String,
    pub traceback: String,
}

impl From<PyErr> for PyNodeConfigError {
    fn from(e: PyErr) -> Self {
        Self::Runtime(py_err_traceback(e))
    }
}
impl From<io::Error> for PyNodeConfigError {
    fn from(e: io::Error) -> Self {
        Self::Io(e.to_string())
    }
}

impl Display for PyNodeConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PyNodeConfigError::Runtime(e) => {
                write!(f, "{}\n\n{}", e.error, e.traceback)
            }

            PyNodeConfigError::NoConfig => write!(f, "No config function found"),
            PyNodeConfigError::ConfigReturn(e) => {
                write!(
                    f,
                    "could not parse config function returned value: {}",
                    e.clone()
                )
            }
            PyNodeConfigError::Io(e) => write!(f, "IO Error: {}", e.clone()),
            PyNodeConfigError::Port(e) => write!(f, "Port Config: {:?}", e.to_string()),
            PyNodeConfigError::Parameter(e) => {
                write!(f, "Parameter Config: {:?}", e.to_string())
            }
        }
    }
}

pub fn py_err_traceback(err: PyErr) -> RuntimeErr {
    Python::with_gil(|py| {
        err.print(py);
        let traceback = err
            .traceback(py)
            .map(|t| t.format().unwrap_or_default())
            .unwrap_or_default();

        return RuntimeErr {
            error: err.to_string(),
            traceback: traceback,
        };
    })
}
