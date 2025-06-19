use std::path::{Path, PathBuf};

use py_foray::{py_module::foray, py_node::parse_node};

use data_model::node::{Dict, NodeTemplate, PortType, PrimitiveType};
use pyo3::{Python, ffi::c_str};

#[test]
fn port_enum_string() {
    pyo3::append_to_inittab!(foray);

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
fn default_test_node() -> NodeTemplate {
    NodeTemplate {
        name: "my_node".into(),
        absolute_path: test_path(),
        relative_path: Default::default(),
        inputs: Ok(Dict::default()),
        outputs: Ok(Dict::default()),
        parameters: Ok(Dict::default()),
    }
}

#[test]
fn empty_config() {
    assert_eq!(
        //// Expected
        default_test_node(),
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
    assert_eq!(
        //// Expected
        NodeTemplate {
            inputs: Ok([
                ("a".into(), PortType::Primitive(PrimitiveType::Integer)),
                ("b".into(), PortType::Primitive(PrimitiveType::Float))
            ]
            .into()),
            ..default_test_node()
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
    assert_eq!(
        //// Expected
        NodeTemplate {
            inputs: Ok([(
                "a".into(),
                PortType::Array(
                    Box::new(PortType::Primitive(PrimitiveType::Integer)),
                    vec![Some(1), None, Some(3)]
                )
            ),]
            .into()),
            ..default_test_node()
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
    let inner_type = PortType::Object(
        [
            ("b_b_a".into(), PortType::Primitive(PrimitiveType::Float)),
            (
                "b_b_b".into(),
                PortType::Array(
                    Box::new(PortType::Primitive(PrimitiveType::Float)),
                    vec![Some(3), Some(4), Some(5)],
                ),
            ),
        ]
        .into(),
    );
    assert_eq!(
        //// Expected
        NodeTemplate {
            outputs: Ok([(
                "b".into(),
                PortType::Object(
                    [
                        ("b_a".into(), PortType::Primitive(PrimitiveType::Integer)),
                        (
                            "b_b".into(),
                            PortType::Array(Box::new(inner_type), vec![Some(1), Some(2), Some(3)],)
                        )
                    ]
                    .into()
                )
            )]
            .into()),
            ..default_test_node()
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
