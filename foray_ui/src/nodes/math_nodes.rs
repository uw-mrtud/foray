use crate::{gui_node::PortDataReference, nodes::NodeError, StableMap};
use ndarray::ArrayD;
use numpy::IxDyn;

use super::port::PortData;

#[allow(clippy::type_complexity)]
pub fn binary_operation(
    inputs: StableMap<String, PortDataReference>,
    f: Box<dyn Fn(&ArrayD<f64>, &ArrayD<f64>) -> ArrayD<f64>>,
) -> Result<StableMap<String, PortData>, NodeError> {
    let a = inputs.get("a").ok_or(NodeError::input_error("a"))?;
    let b = inputs.get("b").ok_or(NodeError::input_error("b"))?;

    let out = match (&**a, &**b) {
        (PortData::ArrayReal(a), PortData::ArrayReal(b)) => f(a, b),
        (PortData::Real(a), PortData::Real(b)) => f(
            &ArrayD::from_shape_simple_fn(IxDyn(&[1]), || *a),
            &ArrayD::from_shape_simple_fn(IxDyn(&[1]), || *b),
        ),
        (PortData::ArrayReal(a), PortData::Real(b)) => {
            f(a, &ArrayD::from_shape_simple_fn(IxDyn(&[1]), || *b))
        }
        _ => panic!("bad inputs!"),
    };

    Ok([("out".into(), PortData::ArrayReal(out))].into())
}
#[allow(clippy::type_complexity)]
pub fn unary_operation(
    inputs: StableMap<String, PortDataReference>,
    f: Box<dyn Fn(&ArrayD<f64>) -> ArrayD<f64>>,
) -> Result<StableMap<String, PortData>, NodeError> {
    let out = match &**inputs.get("a").ok_or(NodeError::input_error("a"))? {
        PortData::ArrayReal(a) => f(a),
        PortData::Real(a) => f(&ArrayD::from_shape_simple_fn(IxDyn(&[1]), || *a)),
        _ => panic!("bad inputs!"),
    };

    Ok([("out".into(), PortData::ArrayReal(out))].into())
}
