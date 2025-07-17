use std::path::{Path, PathBuf};

use foray_data_model::node::{ForayArray, PortData};
use foray_py::{
    py_module::foray,
    py_node::{PyNodeTemplate, parse_node, py_compute_unlocked},
};
use numpy::IxDyn;

use pyo3::{IntoPyObject, Python, py_run};

#[test]
fn port_primitive() {
    pyo3::append_to_inittab!(foray);
    pyo3::prepare_freethreaded_python();

    Python::with_gil(|py| {
        let port_float = PortData::Float(1.5).into_pyobject(py).unwrap();
        let port_integer = PortData::Integer(3).into_pyobject(py).unwrap();
        let port_bool = PortData::Boolean(true).into_pyobject(py).unwrap();
        let port_string = PortData::String("hello".into()).into_pyobject(py).unwrap();
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
        let array_float = ForayArray::Float(
            numpy::ndarray::ArrayD::from_shape_vec(IxDyn(&[3]), vec![1.0, 2.0, 3.0]).unwrap(),
        )
        .into_pyobject(py)
        .unwrap();
        let array2 = ForayArray::Integer(
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
        let r = PortData::Integer(10);
        let g = PortData::Integer(20);
        let b = PortData::Integer(30);
        let pixel = PortData::Object([("r".into(), r), ("g".into(), g), ("b".into(), b)].into())
            .into_pyobject(py)
            .unwrap();
        // PrimitiveData::Float(1.5).into_pyobject(py).unwrap();
        py_run!(
            py,
            pixel,
            r#"
        import numpy
        assert pixel["r"] == 10
        assert pixel["g"] == 20
        assert pixel["b"] == 30
    "#
        );
    });
}

fn test_path() -> PathBuf {
    Path::new("/test/my_node.py").to_path_buf()
}
fn create_test_node(path: PathBuf, node_src: String) -> PyNodeTemplate {
    PyNodeTemplate {
        name: "test".to_string(),
        absolute_path: path,
        relative_path: Default::default(),
        config: parse_node(test_path(), node_src.clone()),
    }
}

#[test]
fn basic_compute() {
    pyo3::prepare_freethreaded_python();

    let node_src = r#"
from foray import port 
def config():
    return {
    "inputs": {"a": port.Integer},
    }

def compute(inputs,parameters):
    return {"b":inputs["a"] + 1}
"#
    .to_string();

    let node = create_test_node(test_path(), node_src.clone());
    let inputs = [("a".into(), PortData::Integer(3))].into();
    let out = py_compute_unlocked(&node, node_src, inputs, Default::default()).unwrap();
    assert_eq!(out, [("b".into(), PortData::Integer(4))].into())
}
#[test]
fn array_compute() {
    pyo3::prepare_freethreaded_python();

    let node_src = r#"
from foray import port 
def config():
    return {
        "inputs": {
            "a": (port.Integer,[3]),
            "b": (port.Integer,[3])
        },
    }

def compute(inputs,parameters):
    return {"c":inputs["a"] + inputs["b"]}
"#
    .to_string();

    // let node = parse_node(test_path(), node_src.clone()).unwrap();
    let node = create_test_node(test_path(), node_src.clone());
    let inputs = [
        (
            "a".into(),
            PortData::Array(ForayArray::Integer(
                numpy::ndarray::ArrayD::from_shape_vec(IxDyn(&[3]), vec![1, 2, 3]).unwrap(),
            )),
        ),
        (
            "b".into(),
            PortData::Array(ForayArray::Integer(
                numpy::ndarray::ArrayD::from_shape_vec(IxDyn(&[3]), vec![4, 5, 6]).unwrap(),
            )),
        ),
    ]
    .into();
    let out = py_compute_unlocked(&node, node_src, inputs, Default::default()).unwrap();
    assert_eq!(
        out,
        [(
            "c".into(),
            PortData::Array(ForayArray::Integer(
                numpy::ndarray::ArrayD::from_shape_vec(IxDyn(&[3]), vec![5, 7, 9]).unwrap(),
            )),
        )]
        .into()
    )
}
