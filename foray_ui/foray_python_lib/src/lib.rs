use pyo3::prelude::*;

/// Formats the sum of two numbers as string.
#[pyfunction]
fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
    Ok((a + b).to_string())
}

/// foray python module for creating nodes
#[pymodule]
#[pyo3(name = "_rust_interface")]
// This module gets re-exported from root/python/foray/__init__.py,
// so its memberers are accsessible directly from the `foray` module.
// pyproject.toml must specify `module-name = "foray._rust_interface"`
// see https://www.maturin.rs/#mixed-rustpython-projects
fn foray_rust_interface(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
    Ok(())
}
