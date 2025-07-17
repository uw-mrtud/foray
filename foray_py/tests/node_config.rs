use std::path::{Path, PathBuf};

use foray_py::{
    py_module::foray,
    py_node::{PyConfig, parse_node},
};

use foray_data_model::node::{Dict, PortType};
use pyo3::{Python, ffi::c_str};

#[test]
fn port_enum_string() {
    pyo3::append_to_inittab!(foray);
    pyo3::prepare_freethreaded_python();

    Python::with_gil(|py| {
        assert!(
            Python::run(
                py,
                c_str!(
                    r#"
import foray
assert foray.port.Integer == "Integer";
                "#
                ),
                None,
                None
            )
            .is_ok()
        )
    });
}

fn test_path() -> PathBuf {
    Path::new("/test/my_node.py").to_path_buf()
}
fn default_test_config() -> PyConfig {
    PyConfig {
        inputs: Ok(Dict::default()),
        outputs: Ok(Dict::default()),
        parameters: Ok(Dict::default()),
    }
}

#[test]
fn empty_config() {
    pyo3::prepare_freethreaded_python();
    assert_eq!(
        //// Expected
        default_test_config(),
        //// Calculated
        parse_node(
            test_path(),
            r#"
def config():
    return {
    "inputs": {},
    "outputs": {},
    "parameters": {},
    }
"#
            .to_string(),
        )
        .unwrap()
    );
}
#[test]
fn filled_config() {
    pyo3::prepare_freethreaded_python();
    assert_eq!(
        //// Expected
        PyConfig {
            inputs: Ok([
                ("a".into(), PortType::Integer),
                ("b".into(), PortType::Float)
            ]
            .into()),
            ..default_test_config()
        },
        //// Calculated
        parse_node(
            test_path(),
            r#"
from foray import port 
def config():
    return {
    "inputs": {"a": port.Integer,"b":port.Float},
    }
"#
            .to_string(),
        )
        .unwrap()
    );
}

#[test]
fn array_config() {
    pyo3::prepare_freethreaded_python();
    assert_eq!(
        //// Expected
        PyConfig {
            inputs: Ok([(
                "a".into(),
                PortType::Array(Box::new(PortType::Integer), vec![Some(1), None, Some(3)])
            ),]
            .into()),
            ..default_test_config()
        },
        //// Calculated
        parse_node(
            test_path(),
            r#"
from foray import port 
def config():
    return {
    "inputs": {"a": (port.Integer,[1,None,3])},
    }
"#
            .to_string(),
        )
        .unwrap()
    );
}

#[test]
fn nested_config() {
    pyo3::prepare_freethreaded_python();
    let inner_type = PortType::Object(
        [
            ("b_b_a".into(), PortType::Float),
            (
                "b_b_b".into(),
                PortType::Array(Box::new(PortType::Float), vec![Some(3), Some(4), Some(5)]),
            ),
        ]
        .into(),
    );
    assert_eq!(
        //// Expected
        PyConfig {
            outputs: Ok([(
                "b".into(),
                PortType::Object(
                    [
                        ("b_a".into(), PortType::Integer),
                        (
                            "b_b".into(),
                            PortType::Array(Box::new(inner_type), vec![Some(1), Some(2), Some(3)],)
                        )
                    ]
                    .into()
                )
            )]
            .into()),
            ..default_test_config()
        },
        //// Calculated
        parse_node(
            test_path(),
            r#"
from foray import port
def config():
    return {
    "outputs": {"b": {"b_a": port.Integer,
        "b_b": (
                {
                "b_b_a": port.Float,
                "b_b_b": (port.Float,[3,4,5]) 
                }
                ,[1,2,3]
               )
            }
        }
    }
"#
            .to_string(),
        )
        .unwrap()
    );
}
