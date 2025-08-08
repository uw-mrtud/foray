use std::path::PathBuf;

use pyo3::{
    Python,
    ffi::c_str,
    types::{PyAnyMethods, PyModule},
};

/// Information about a python package that is provides nodes
/// Unprocessed, strait from python
pub struct RawNodePackageInfo {
    // The package's name
    pub package_name: String,
    // the absolute python path where nodes are defined
    pub entry_point: String,
    // The installed location where nodes are defined
    pub abs_path: PathBuf,
    // the absolute python path of each node
    pub node_py_paths: Vec<String>,
}

pub fn get_foray_py_packages() -> Vec<RawNodePackageInfo> {
    Python::with_gil(|py| {
        let snippet = PyModule::from_code(
            py,
            c_str!(
                r#"
from importlib.metadata import entry_points
import pkgutil


def get_node_paths():
    # get all 'foray' entry points as configured in `pyproject.toml`s
    # (module, entry_point/node_module)
    foray_modules = [(ep.load(), ep.value) for ep in entry_points(group="foray")]

    def recurse_module(module):
        return pkgutil.walk_packages(module.__path__, module.__name__ + ".")

    return [
        (
            m.__name__.split(".")[0],  # get root module
            entry_point,
            # Usually there is just 1 element in this list. Unsure when there is more
            m.__path__[0],
            # get all submodules
            [
                name
                for _, name, ispkg in recurse_module(m)
                if not ispkg  # discard packages, e.g. __init__.py
            ],
        )
        for (m, entry_point) in foray_modules
    ]
"#
            ),
            c_str!("dicover.py"),
            c_str!("discover"),
        )
        .unwrap();
        let result: Vec<(String, String, PathBuf, Vec<String>)> = snippet
            .getattr("get_node_paths")
            .unwrap()
            .call0()
            .unwrap()
            .extract()
            .unwrap();

        result
            .into_iter()
            .map(
                |(package_name, entry_point, abs_path, node_py_paths)| RawNodePackageInfo {
                    package_name,
                    entry_point,
                    abs_path,
                    node_py_paths,
                },
            )
            .collect()
    })
}
