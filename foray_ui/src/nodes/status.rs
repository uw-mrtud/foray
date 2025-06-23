use derive_more::derive::Display;
use iced::widget::{text, Text};
use serde::{Deserialize, Serialize};
use std::{error, time::Instant};

use crate::style::icon::icon;

#[derive(Clone, Debug, Default, Display, PartialEq, Eq)]
pub enum NodeStatus {
    #[default]
    Idle,
    #[display("Running")]
    Running(Instant),
    Error(NodeError),
}

impl NodeStatus {
    pub fn icon(&'_ self) -> Text<'_> {
        match self {
            NodeStatus::Idle => icon(""),
            NodeStatus::Running(_) => icon(""), //icon(""),
            NodeStatus::Error(_) => icon("").style(text::danger),
        }
    }

    pub fn text_element(&'_ self) -> Text<'_> {
        match self {
            NodeStatus::Idle => text(""),
            NodeStatus::Running(_) => text(""),
            NodeStatus::Error(err) => text(err.to_string()).style(text::danger),
        }
    }
}

//TODO: Cleanup errors and make them more discrete where possible
#[derive(Debug, Display, Clone, Serialize, Deserialize, Default, PartialEq, Eq, PartialOrd)]
pub enum NodeError {
    Input(String),
    Output(String),
    Config(String),
    Syntax(String),
    FileSys(String),
    Runtime(String),
    MissingCompute(String),
    #[default]
    Other,
}
impl NodeError {
    pub fn input_error(input_name: impl Into<String>) -> NodeError {
        NodeError::Input(format!("Input '{:}' not found", input_name.into()))
    }
}

impl error::Error for NodeError {}
