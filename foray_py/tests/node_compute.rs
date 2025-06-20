use foray_data_model::node::{PortData, PrimitiveArray, PrimitiveData};
use foray_py::py_module::foray;
use numpy::IxDyn;

use pyo3::{IntoPyObject, Python, py_run};
#[test]
fn port_primitive() {
    pyo3::append_to_inittab!(foray);
    pyo3::prepare_freethreaded_python();

    Python::with_gil(|py| {
        let port_float = PrimitiveData::Float(1.5).into_pyobject(py).unwrap();
        let port_integer = PrimitiveData::Integer(3).into_pyobject(py).unwrap();
        let port_bool = PrimitiveData::Boolean(true).into_pyobject(py).unwrap();
        let port_string = PrimitiveData::String("hello".into())
            .into_pyobject(py)
            .unwrap();
        py_run!(
            py,
            port_float
            port_integer
            port_bool
            port_string,
            r#"
        assert port_float == 1.5 
        assert port_integer == 3
        assert port_bool == True 
        assert port_string == "hello" 
    "#
        );
    });
}
#[test]
fn port_array() {
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        let array_float = PrimitiveArray::Float(
            numpy::ndarray::ArrayD::from_shape_vec(IxDyn(&[3]), vec![1.0, 2.0, 3.0]).unwrap(),
        )
        .into_pyobject(py)
        .unwrap();
        let array2 = PrimitiveArray::Integer(
            numpy::ndarray::ArrayD::from_shape_vec(IxDyn(&[2, 2]), vec![1, 2, 3, 4]).unwrap(),
        )
        .into_pyobject(py)
        .unwrap();
        // PrimitiveData::Float(1.5).into_pyobject(py).unwrap();
        py_run!(
            py,
            array_float array2,
            r#"
        import numpy
        assert array_float[0] == 1.0
        assert array_float[1] == 2.0
        assert array_float[2] == 3.0

        assert array2[0,0] == 1
        assert array2[0,1] == 2
        assert array2[1,0] == 3
        assert array2[1,1] == 4
    "#
        );
    });
}
#[test]
fn port_object() {
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        let r = PortData::Primitve(PrimitiveData::Integer(10));
        let g = PortData::Primitve(PrimitiveData::Integer(20));
        let b = PortData::Primitve(PrimitiveData::Integer(30));
        let pixel = PortData::Object([("r".into(), r), ("g".into(), g), ("b".into(), b)].into())
            .into_pyobject(py)
            .unwrap();
        // PrimitiveData::Float(1.5).into_pyobject(py).unwrap();
        py_run!(
            py,
            pixel,
            r#"
        import numpy
        print(pixel["r"])
        assert pixel["r"] == 10
        assert pixel["g"] == 20
        assert pixel["b"] == 30
    "#
        );
    });
}
