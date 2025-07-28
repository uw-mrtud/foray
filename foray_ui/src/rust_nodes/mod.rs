use derive_more::Display;
use foray_data_model::{
    node::{Dict, NodeError, PortData, PortType},
    WireDataContainer, WireDataReference,
};
use foray_graph::graph::{ForayNodeError, GraphNode};
use serde::{Deserialize, Serialize};
use strum::{EnumIter, VariantNames};

#[derive(
    Clone, Debug, Display, EnumIter, VariantNames, Serialize, Deserialize, PartialEq, PartialOrd,
)]
pub enum RustNodeTemplate {
    Identity,
    Constant(f64),
    Add,
    Subtract,
    Multiply,
    Divide,
    Cos,
    Sin,
    Sinc,
    // #[display("Linspace")]
    // Linspace(LinspaceConfig),
    // #[display("Plot")]
    // Plot(Plot),
    // #[display("Plot2D")]
    // Plot2D(Plot2D),
    // #[display("VectorField")]
    // VectorField(VectorField),
}

// #[derive(Clone, Debug, Serialize, Deserialize, PartialEq, PartialOrd)]
// pub enum UINodeTemplate {
//     #[debug("{_0:?}")]
//     RustNode(RustNode),
//     #[debug("{_0:?}")]
//     PyNode(PyNodeTemplate),
// }
//
// // impl From<NodeData> for UINodeTemplate {
// //     fn from(value: NodeData) -> Self {
// //         value.template
// //     }
// // }
// // impl From<UINodeTemplate> for NodeData {
// //     fn from(template: UINodeTemplate) -> Self {
// //         NodeData {
// //             template,
// //             status: NodeStatus::Idle,
// //             run_time: None,
// //         }
// //     }
// // }
// impl UINodeTemplate {
//     pub fn duplicate(&self) -> Self {
//         match self {
//             // NodeTemplate::RustNode(RustNode::Plot2D(plot2d)) => {
//             //     NodeTemplate::RustNode(RustNode::Plot2D(Plot2D {
//             //         image_handle: None,
//             //         ..plot2d.clone()
//             //     }))
//             // }
//             _ => self.clone(),
//         }
//     }
// }

impl RustNodeTemplate {
    fn fallible_compute(
        &mut self,
        inputs: Dict<String, WireDataReference<PortData>>,
    ) -> Result<Dict<String, PortData>, ForayNodeError> {
        Ok(match self {
            RustNodeTemplate::Identity => [(
                "out".to_string(),
                (**inputs
                    .get("a")
                    .ok_or(ForayNodeError::NodeError(NodeError::Input(
                        "input 'a' not found".into(),
                    )))?)
                .clone(),
            )]
            .into(),
            RustNodeTemplate::Constant(value) => {
                [("out".to_string(), PortData::Float(*value))].into()
            }
            _ => [].into(), // RustNode::Add => binary_operation(inputs, Box::new(|a, b| a + b))?,
                            // RustNode::Subtract => binary_operation(inputs, Box::new(|a, b| a - b))?,
                            // RustNode::Multiply => binary_operation(inputs, Box::new(|a, b| a * b))?,
                            //
                            // RustNode::Divide => binary_operation(inputs, Box::new(|a, b| a / b))?,
                            //
                            // RustNode::Cos => unary_operation(inputs, Box::new(|a| a.cos()))?,
                            //
                            // RustNode::Sin => unary_operation(inputs, Box::new(|a| a.sin()))?,
                            //
                            // RustNode::Sinc => unary_operation(
                            //     inputs,
                            //     Box::new(|a| {
                            //         a.map(|x| match x {
                            //             0. => 1.,
                            //             _ => x.sin() / x,
                            //         })
                            //     }),
                            // )?,
                            //
                            // RustNode::Linspace(linspace_config) => linspace_config.compute(inputs),
                            // RustNode::Plot(_) => [].into(),
                            // RustNode::Plot2D(plot_2d) => {
                            //     let out = plot_2d.input_changed(inputs);
                            //     [("out".into(), out)].into()
                            // }
                            // RustNode::VectorField(_) => [].into(),
        })
    }
}
// impl RustNode {
//     /// A node can produce any number of "templates" which will be used to populate the
//     /// list of selectable new nodes that can be created.
//     /// Notably, PyNode will produce a dynamic number of nodes,
//     /// depending on what nodes are found in the filesystem at runtime.
//     pub fn template_variants(&self) -> ForayNode {
//         ForayNode::RustNode(self.clone()).into()
//     }
// }

impl GraphNode<PortType, PortData> for RustNodeTemplate {
    fn inputs(&self) -> Dict<String, PortType> {
        let prim_float = PortType::Float;
        let binary_in = [
            ("a".to_string(), prim_float.clone()),
            ("b".to_string(), prim_float.clone()),
        ]
        .into();
        let unary_in = [("a".to_string(), prim_float.clone())].into();

        match &self {
            RustNodeTemplate::Identity => [("a".to_string(), prim_float)].into(),
            RustNodeTemplate::Constant(_constant_node) => [].into(),
            RustNodeTemplate::Add => binary_in,
            RustNodeTemplate::Subtract => binary_in,
            RustNodeTemplate::Multiply => binary_in,
            RustNodeTemplate::Divide => binary_in,
            RustNodeTemplate::Cos => unary_in,
            RustNodeTemplate::Sin => unary_in,
            RustNodeTemplate::Sinc => unary_in,
            // RustNode::Linspace(_) => [].into(),
            // RustNode::Plot(_) => [
            //     ("x".to_string(), PortType::Real),
            //     ("y".to_string(), PortType::Real),
            // ]
            // .into(),
            // RustNode::Plot2D(_) => [("a".to_string(), PortType::ArrayReal)].into(),
            // RustNode::VectorField(_) => [("a".to_string(), PortType::ArrayReal)].into(),
        }
    }

    fn outputs(&self) -> Dict<String, PortType> {
        let prim_float = PortType::Float;
        let real_out = [("out".to_string(), prim_float)].into();
        // let array_out = [("out".to_string(), PortType::ArrayReal)].into();
        match self {
            RustNodeTemplate::Identity => real_out,
            RustNodeTemplate::Constant(_constant_node) => real_out,
            RustNodeTemplate::Add => real_out,
            RustNodeTemplate::Subtract => real_out,
            RustNodeTemplate::Multiply => real_out,
            RustNodeTemplate::Divide => real_out,
            RustNodeTemplate::Cos => real_out,
            RustNodeTemplate::Sin => real_out,
            RustNodeTemplate::Sinc => real_out,
            //     RustNode::Linspace(_) => real_out,
            //     RustNode::Plot(_) => array_out,
            //     RustNode::Plot2D(_) => array_out,
            //     RustNode::VectorField(_) => [].into(),
        }
    }

    fn compute(
        mut self,
        inputs: Dict<String, WireDataContainer<PortData>>,
    ) -> Result<Dict<String, PortData>, ForayNodeError> {
        // unpack mutex

        // match &mut self.template {
        //     UINodeTemplate::RustNode(rust_node) => {
        let data = inputs
            .keys()
            .map(|k| (k.clone(), inputs[k].read().unwrap()))
            .collect();
        self.fallible_compute(data)
        // }
        // UINodeTemplate::PyNode(py_node) => match py_node.clone().compute(inputs) {
        //     Ok((calculated_ports, new_py_node)) => Ok((
        //         calculated_ports,
        //         NodeData {
        //             template: UINodeTemplate::PyNode(new_py_node),
        //             ..self
        //         },
        //     )),
        //     Err(e) => Err(e),
        // },
    }
}
