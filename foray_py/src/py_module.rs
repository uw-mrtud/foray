use pyo3::prelude::*;

// #[pyclass]
// struct PyPrimitive(PrimitiveType);

#[pymodule]
pub fn foray(py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    //m.add_function(wrap_pyfunction!(get_build_time, m)?)?;
    // m.add_function(wrap_pyfunction!(slider, m)?)?;
    // m.add_class::<Ui>()?;

    let port_submodule = PyModule::new(py, "port")?;
    port_submodule.add("Integer", "Integer")?;
    port_submodule.add("Float", "Float")?;
    port_submodule.add("ArrayComplex", "ArrayComplex")?;
    port_submodule.add("ArrayReal", "ArrayReal")?;
    port_submodule.add("Dynamic", "Dynamic")?;
    m.add_submodule(&port_submodule)?;
    // let _ = m.add_class::<PrimitiveType>();

    Ok(())
}
