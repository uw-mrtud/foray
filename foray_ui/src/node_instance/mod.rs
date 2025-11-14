pub mod histogram;
pub(super) mod value_mapping;
pub mod visualization_parameters;
pub mod visualiztion;

use std::time::Instant;

use derive_more::derive::Debug;
use foray_data_model::{
    node::{Dict, PortData, PortType},
    WireDataContainer,
};
use foray_graph::graph::{ForayNodeError, GraphNode, PortName};
use foray_py::py_node::{py_compute, PyNodeTemplate};
use serde::{Deserialize, Serialize};
use visualiztion::Visualization;

use crate::rust_nodes::RustNodeTemplate;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum ForayNodeTemplate {
    RustNode(RustNodeTemplate),
    PyNode(PyNodeTemplate),
}

impl ForayNodeTemplate {
    pub fn default_parameters(&self) -> Dict<String, PortData> {
        match &self {
            ForayNodeTemplate::RustNode(_rust_node) => Default::default(),
            ForayNodeTemplate::PyNode(py_node) => py_node
                .parameters()
                .unwrap_or_default()
                .iter()
                .map(|(k, v)| (k.clone(), v.default_value()))
                .collect(),
        }
    }
    pub fn name(&self) -> String {
        match &self {
            ForayNodeTemplate::RustNode(rust_node) => rust_node.to_string(),
            ForayNodeTemplate::PyNode(py_node) => py_node.name.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd)]
pub enum NodeStatus {
    Idle { last_finished: Option<Instant> },
    Running { start: Instant },
    Error(Vec<ForayNodeError>),
}

impl Default for NodeStatus {
    fn default() -> Self {
        NodeStatus::Idle {
            last_finished: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ForayNodeInstance {
    // TODO: should this be just an identifier, and keep NodeTemplates
    // in one spot, refering to them when necessary?
    pub template: ForayNodeTemplate,
    pub parameters_values: Dict<String, PortData>,
    #[serde(default)]
    pub visualization: Visualization,
    #[serde(skip)]
    // If there are errors for any of NodeDefinition fields, the field will be empty,
    // The error will be noted in NodeStatus
    pub status: NodeStatus,
}

impl GraphNode<PortType, PortData> for ForayNodeInstance {
    fn inputs(&self) -> Dict<PortName, PortType> {
        match &self.template {
            ForayNodeTemplate::RustNode(rust_node) => rust_node.inputs(),
            ForayNodeTemplate::PyNode(py_node) => py_node.inputs().unwrap_or_default(),
        }
    }

    fn outputs(&self) -> Dict<PortName, PortType> {
        match &self.template {
            ForayNodeTemplate::RustNode(rust_node) => rust_node.outputs(),
            ForayNodeTemplate::PyNode(py_node) => py_node.outputs().unwrap_or_default(),
        }
    }

    fn compute(
        self,
        inputs: Dict<PortName, WireDataContainer<PortData>>,
    ) -> Result<Dict<PortName, PortData>, ForayNodeError> {
        match self.template {
            ForayNodeTemplate::RustNode(rust_node) => rust_node.compute(inputs),
            ForayNodeTemplate::PyNode(py_node) => {
                py_compute(&py_node, inputs, self.parameters_values)
                    .map_err(ForayNodeError::PyNodeConifgError)
            }
        }
    }
}

impl PartialOrd for ForayNodeInstance {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.template.partial_cmp(&other.template) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.status.partial_cmp(&other.status)
    }
}
impl From<ForayNodeTemplate> for ForayNodeInstance {
    fn from(template: ForayNodeTemplate) -> Self {
        ForayNodeInstance {
            parameters_values: template.default_parameters(),
            visualization: Default::default(),
            status: match &template {
                ForayNodeTemplate::RustNode(_rust_node_template) => Default::default(),
                ForayNodeTemplate::PyNode(py_node_template) => {
                    let errors: Vec<_> = py_node_template
                        .errors()
                        .into_iter()
                        .map(ForayNodeError::PyNodeConifgError)
                        .collect();
                    if errors.is_empty() {
                        Default::default()
                    } else {
                        NodeStatus::Error(errors)
                    }
                }
            },
            template,
        }
    }
}
