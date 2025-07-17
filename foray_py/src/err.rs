use std::{fmt::Display, io};

use foray_data_model::node::{ParameterError, PortError};
use log::warn;
use pyo3::{PyErr, Python, types::PyTracebackMethods};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Serialize, Deserialize)]
pub enum PyNodeConfigError {
    Runtime(String),
    NoConfig,
    ConfigReturn(String),
    Port(PortError),
    Parameter(ParameterError),
    Io(String),
}

impl From<PyErr> for PyNodeConfigError {
    fn from(e: PyErr) -> Self {
        Self::Runtime(e.to_string())
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
                write!(f, "Python Runtime Error: \n{}", e.clone())
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

pub fn py_err_traceback(err: PyErr) -> String {
    Python::with_gil(|py| {
        let (file, line_number): (String, String) = err
            .traceback(py)
            .map(|t| {
                let formatted_trace = t.format().unwrap_or_default();
                warn!("{}", formatted_trace);
                let words: Vec<_> = formatted_trace.split(" ").collect();
                let file = words
                    .iter()
                    .position(|s| *s == "File")
                    .map(|pos| words[pos + 1].split('"').collect::<Vec<_>>()[1])
                    .unwrap_or("FILE_NOT_FOUND");

                let line = words
                    .iter()
                    .position(|s| *s == "line")
                    .map(|pos| words[pos + 1].split(",").next().unwrap_or_default())
                    .unwrap_or("LINE_NOT_FOUND");
                (file.to_string(), line.to_string())
            })
            .unwrap_or_default();
        format!("File: {file}, line {line_number}\n {}", err)
    })
}
