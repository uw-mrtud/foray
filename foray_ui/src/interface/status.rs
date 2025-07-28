use iced::widget::{text, Text};
use itertools::Itertools;

use crate::{node_instance::NodeStatus, style::icon::icon};

pub fn node_status_icon(status: &'_ NodeStatus) -> Text<'_> {
    match status {
        NodeStatus::Idle => icon(""),
        NodeStatus::Running { start: _ } => icon(""),
        NodeStatus::Error(_py_node_error) => icon("").style(text::danger),
    }
}

pub fn node_status_text_element(status: &'_ NodeStatus) -> Text<'_> {
    match status {
        NodeStatus::Idle => text(""),
        NodeStatus::Running { start: _ } => text(""),
        NodeStatus::Error(err) => {
            text(err.iter().map(|e| e.to_string()).join("\n")).style(text::danger)
        }
    }
}

// //TODO: Cleanup errors and make them more discrete where possible
// #[derive(Debug, Display, Clone, Serialize, Deserialize, Default, PartialEq, Eq, PartialOrd)]
// pub enum NodeError {
//     Input(String),
//     Output(String),
//     Config(String),
//     Syntax(String),
//     FileSys(String),
//     Runtime(String),
//     MissingCompute(String),
//     #[default]
//     Other,
// }
// impl NodeError {
//     pub fn input_error(input_name: impl Into<String>) -> NodeError {
//         NodeError::Input(format!("Input '{:}' not found", input_name.into()))
//     }
// }
//
// impl error::Error for NodeError {}
