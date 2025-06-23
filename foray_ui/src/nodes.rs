use std::time::Duration;

pub mod constant;
pub mod linspace;
pub mod math_nodes;
pub mod plot;
pub mod plot_complex;
pub mod port;
pub mod status;
pub mod vector_field;

use crate::app::Message;
use crate::graph::GraphNode;
use crate::gui_node::{GUINode, PortDataContainer, PortDataReference};
use crate::interface::node::default_node_size;
use crate::nodes::linspace::LinspaceConfig;
use crate::nodes::math_nodes::{binary_operation, unary_operation};
use crate::nodes::plot::Plot;
use crate::nodes::plot_complex::Plot2D;
use crate::python::py_node::PyNode;
use crate::StableMap;
use derive_more::derive::{Debug, Display};
use iced::widget::text;
use iced::{Font, Size};
use port::{PortData, PortType};
use serde::{Deserialize, Serialize};
use status::{NodeError, NodeStatus};
use strum::{EnumIter, VariantNames};
use vector_field::VectorField;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeData {
    pub template: NodeTemplate,
    #[serde(skip)]
    pub status: NodeStatus,
    #[serde(skip)]
    pub run_time: Option<Duration>,
}

#[derive(
    Clone, Debug, Display, EnumIter, VariantNames, Serialize, Deserialize, PartialEq, PartialOrd,
)]
pub enum RustNode {
    Identity,
    Constant(f64),
    Add,
    Subtract,
    Multiply,
    Divide,
    Cos,
    Sin,
    Sinc,
    #[display("Linspace")]
    Linspace(LinspaceConfig),
    #[display("Plot")]
    Plot(Plot),
    #[display("Plot2D")]
    Plot2D(Plot2D),
    #[display("VectorField")]
    VectorField(VectorField),
}

#[derive(Clone, Debug, Display, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum NodeTemplate {
    #[debug("{_0:?}")]
    RustNode(RustNode),
    #[debug("{_0:?}")]
    PyNode(PyNode),
}

impl From<NodeData> for NodeTemplate {
    fn from(value: NodeData) -> Self {
        value.template
    }
}
impl From<NodeTemplate> for NodeData {
    fn from(template: NodeTemplate) -> Self {
        NodeData {
            template,
            status: NodeStatus::Idle,
            run_time: None,
        }
    }
}
impl NodeTemplate {
    pub fn duplicate(&self) -> Self {
        match self {
            NodeTemplate::RustNode(RustNode::Plot2D(plot2d)) => {
                NodeTemplate::RustNode(RustNode::Plot2D(Plot2D {
                    image_handle: None,
                    ..plot2d.clone()
                }))
            }
            _ => self.clone(),
        }
    }
}

impl NodeData {
    fn fallible_compute(
        &mut self,
        inputs: StableMap<String, PortDataReference>,
    ) -> Result<(StableMap<String, PortData>, NodeData), NodeError> {
        Ok((
            match &mut self.template {
                NodeTemplate::RustNode(rust_node) => match rust_node {
                    RustNode::Identity => [(
                        "out".to_string(),
                        (**inputs
                            .get("a")
                            .ok_or(NodeError::Input("input 'a' not found".to_string()))?)
                        .clone(),
                    )]
                    .into(),
                    RustNode::Constant(value) => {
                        [("out".to_string(), PortData::Real(*value))].into()
                    }
                    RustNode::Add => binary_operation(inputs, Box::new(|a, b| a + b))?,
                    RustNode::Subtract => binary_operation(inputs, Box::new(|a, b| a - b))?,
                    RustNode::Multiply => binary_operation(inputs, Box::new(|a, b| a * b))?,

                    RustNode::Divide => binary_operation(inputs, Box::new(|a, b| a / b))?,

                    RustNode::Cos => unary_operation(inputs, Box::new(|a| a.cos()))?,

                    RustNode::Sin => unary_operation(inputs, Box::new(|a| a.sin()))?,

                    RustNode::Sinc => unary_operation(
                        inputs,
                        Box::new(|a| {
                            a.map(|x| match x {
                                0. => 1.,
                                _ => x.sin() / x,
                            })
                        }),
                    )?,

                    RustNode::Linspace(linspace_config) => linspace_config.compute(inputs),
                    RustNode::Plot(_) => [].into(),
                    RustNode::Plot2D(plot_2d) => {
                        let out = plot_2d.input_changed(inputs);
                        [("out".into(), out)].into()
                    }
                    RustNode::VectorField(_) => [].into(),
                },

                NodeTemplate::PyNode(py_node) => py_node.compute(inputs)?,
            },
            self.clone(),
        ))
    }
}
impl RustNode {
    /// A node can produce any number of "templates" which will be used to populate the
    /// list of selectable new nodes that can be created.
    /// Notably, PyNode will produce a dynamic number of nodes,
    /// depending on what nodes are found in the filesystem at runtime.
    pub fn template_variants(&self) -> NodeData {
        NodeTemplate::RustNode(self.clone()).into()
    }
}

impl GraphNode<NodeData, PortType, PortData> for NodeData {
    fn inputs(&self) -> StableMap<String, PortType> {
        let binary_in = [
            ("a".to_string(), PortType::Real),
            ("b".to_string(), PortType::Real),
        ]
        .into();
        let unary_in = [("a".to_string(), PortType::Real)].into();

        match &self.template {
            NodeTemplate::RustNode(rn) => match rn {
                RustNode::Identity => [("a".to_string(), PortType::Real)].into(),
                RustNode::Constant(_constant_node) => [].into(),
                RustNode::Add => binary_in,
                RustNode::Subtract => binary_in,
                RustNode::Multiply => binary_in,
                RustNode::Divide => binary_in,
                RustNode::Cos => unary_in,
                RustNode::Sin => unary_in,
                RustNode::Sinc => unary_in,
                RustNode::Linspace(_) => [].into(),
                RustNode::Plot(_) => [
                    ("x".to_string(), PortType::Real),
                    ("y".to_string(), PortType::Real),
                ]
                .into(),
                RustNode::Plot2D(_) => [("a".to_string(), PortType::ArrayReal)].into(),
                RustNode::VectorField(_) => [("a".to_string(), PortType::ArrayReal)].into(),
            },
            NodeTemplate::PyNode(py_node) => py_node.ports.clone().unwrap_or_default().inputs,
        }
    }

    fn outputs(&self) -> StableMap<String, PortType> {
        let real_out = [("out".to_string(), PortType::Real)].into();
        let array_out = [("out".to_string(), PortType::ArrayReal)].into();
        match &self.template {
            NodeTemplate::RustNode(rn) => match rn {
                RustNode::Identity => real_out,
                RustNode::Constant(_constant_node) => real_out,
                RustNode::Add => real_out,
                RustNode::Subtract => real_out,
                RustNode::Multiply => real_out,
                RustNode::Divide => real_out,
                RustNode::Cos => real_out,
                RustNode::Sin => real_out,
                RustNode::Sinc => real_out,
                RustNode::Linspace(_) => real_out,
                RustNode::Plot(_) => array_out,
                RustNode::Plot2D(_) => array_out,
                RustNode::VectorField(_) => [].into(),
            },
            NodeTemplate::PyNode(py_node) => py_node.ports.clone().unwrap_or_default().outputs,
        }
    }

    fn compute(
        mut self,
        inputs: StableMap<String, PortDataContainer>,
    ) -> Result<(StableMap<String, PortData>, NodeData), NodeError> {
        // unpack mutex
        let data = inputs
            .keys()
            .map(|k| (k.clone(), inputs[k].read().unwrap()))
            .collect();

        self.fallible_compute(data)
    }
}

impl GUINode for NodeTemplate {
    fn name(&self) -> String {
        match &self {
            NodeTemplate::RustNode(rn) => match rn {
                RustNode::Identity => "Identity".to_string(),
                RustNode::Constant(_value) => "Constant".to_string(),
                RustNode::Add => "Add".to_string(),
                RustNode::Subtract => "Subtract".to_string(),
                RustNode::Multiply => "Multiply".to_string(),
                RustNode::Divide => "Divide".to_string(),
                RustNode::Cos => "cos".to_string(),
                RustNode::Sin => "sin".to_string(),
                RustNode::Sinc => "sinc".to_string(),
                RustNode::Linspace(_linspace_config) => "Linspace".to_string(),
                RustNode::Plot(_) => "Plot".to_string(),
                RustNode::Plot2D(_) => "Plot 2D".to_string(),
                RustNode::VectorField(_) => "Plot Vector Field".to_string(),
            },
            NodeTemplate::PyNode(py_node) => py_node
                .absolute_path
                .file_stem()
                .map(|s| s.to_string_lossy())
                .unwrap_or(("NOT_FOUND").into())
                .into(),
        }
    }

    fn view(
        &'_ self,
        id: u32,
        input_data: StableMap<String, PortDataContainer>,
    ) -> iced::Element<'_, Message> {
        let operation = |s| {
            text(s)
                .font(Font::with_name("DejaVu Math TeX Gyre"))
                .size(30)
                .into()
        };
        let trig = |s| {
            text(s)
                .size(20)
                .font(Font::with_name("DejaVu Math TeX Gyre"))
                .into()
        };

        match self {
            NodeTemplate::RustNode(rn) => match rn {
                RustNode::Constant(value) => constant::view(id, *value),
                RustNode::Linspace(linspace_config) => linspace_config.view(id),
                RustNode::Plot(plot) => plot.view(id, input_data),
                RustNode::Plot2D(plot) => plot.view(id, input_data),
                RustNode::VectorField(vf) => vf.view(id, input_data),
                RustNode::Add => operation("+"),
                RustNode::Subtract => operation("−"),
                RustNode::Multiply => operation("×"),
                RustNode::Divide => operation("÷"),
                RustNode::Cos => trig("cos(α)"),
                RustNode::Sin => trig("sin(α)"),
                RustNode::Sinc => trig("sinc(α)"),

                _ => text(self.name()).into(),
            },
            NodeTemplate::PyNode(_) => text(self.name()).into(),
        }
    }
    fn node_size(&self) -> iced::Size {
        let dft = default_node_size();
        match self {
            NodeTemplate::RustNode(rn) => match rn {
                RustNode::Linspace(_) => Size::new(dft.width * 2., dft.height),
                RustNode::Plot(_) => dft * 2.,
                RustNode::Plot2D(_) => (dft.width * 2., dft.width * 2.).into(),
                RustNode::VectorField(_) => (dft.width * 2., dft.width * 2.).into(),
                _ => dft,
            },
            NodeTemplate::PyNode(_) => dft,
        }
    }

    fn config_view(
        &'_ self,
        id: u32,
        input_data: StableMap<String, PortDataContainer>,
    ) -> Option<iced::Element<'_, Message>> {
        match &self {
            NodeTemplate::RustNode(rn) => match rn {
                RustNode::Plot(plot) => plot.config_view(id, input_data),
                RustNode::Plot2D(plot) => plot.config_view(id, input_data),
                RustNode::VectorField(plot) => plot.config_view(id, input_data),
                _ => None,
            },
            NodeTemplate::PyNode(pn) => pn.config_view(id, input_data),
        }
    }
}
