use foray_graph::graph::ForayNodeError;
use iced::{
    widget::{column, container, container::rounded_box, row, text, tooltip, tooltip::Position},
    Alignment::Center,
    Element,
};

use crate::{node_instance::NodeStatus, style::icon::icon};

pub fn node_status_widget<'a, M: 'a>(status: &'a NodeStatus) -> Element<'a, M> {
    match status {
        NodeStatus::Idle => text("").into(),
        NodeStatus::Running { start: _ } => text("").into(),
        NodeStatus::Error(errs) => column(errs.iter().map(|e| {
            let (summary, detailed) = match e {
                ForayNodeError::PyNodeConifgError(py_node_config_error) => {
                    match py_node_config_error {
                        foray_py::err::PyNodeConfigError::Runtime(runtime_err) => (
                            runtime_err.error.clone(),
                            Some(runtime_err.traceback.clone()),
                        ),
                        _ => (py_node_config_error.to_string(), None),
                    }
                }
                ForayNodeError::NodeError(node_error) => (node_error.to_string(), None),
            };

            let summary_row = row![
                icon("").style(text::danger),
                text(summary).style(text::danger).size(10)
            ]
            .spacing(8)
            .align_y(Center);

            match detailed {
                Some(d) => tooltip(
                    summary_row,
                    container(text(d).size(10)).padding(2).style(rounded_box),
                    Position::Right,
                )
                .into(),
                None => summary_row.into(),
            }
        }))
        .into(),
    }
}
