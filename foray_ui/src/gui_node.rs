use foray_data_model::{
    node::{Dict, PortData, UIParameter},
    WireDataContainer,
};
use foray_graph::{
    node_instance::{ForayNodeInstance, ForayNodeTemplate},
    rust_node::RustNodeTemplate,
};
use iced::{
    widget::{column, horizontal_space, row, slider, text},
    Alignment::Center,
    Element, Font,
    Length::Fill,
};

use crate::{
    app::Message,
    interface::{
        node::default_node_size,
        numeric_input::{self, PartialUIValue},
    },
};
const PRECISION: f64 = 100.0;

// use std::sync::{Arc, RwLock, RwLockReadGuard};
//
// use foray_data_model::node::{NodeStatus, PortData, PortType};
// use foray_graph::graph::Graph;
// use iced::{widget::text, Element};
//
// use crate::{app::Message, nodes::NodeData, StableMap};
//
// pub trait GUINode: derive_more::Debug {
//     //TODO make this more understandable. clearer distinction between graph and gui?
//     // split out port names, and compute logic?
//     //fn network_node(&self) -> GraphNode<PortType, PortData>;
//
//     //TODO: Port validation logic? here, or handled at the portType level?
//     //TODO: conversion logic?
//
//     fn name(&self) -> String;
//
//     fn view(
//         &'_ self,
//         _id: u32,
//         _input_data: StableMap<String, PortDataContainer>,
//     ) -> Element<'_, Message> {
//         text("default").into()
//     }
//     fn node_size(&self) -> iced::Size;
//
//     fn config_view(
//         &self,
//         _id: u32,
//         _input_data: StableMap<String, PortDataContainer>,
//     ) -> Option<Element<'_, Message>> {
//         None
//     }
// }
//
// // pub type PortDataReference<'a> = RwLockReadGuard<'a, PortData>;
// // pub type PortDataContainer = Arc<RwLock<PortData>>;
// // pub type GuiGraph = Graph<NodeData, PortType, PortData>;
//
// pub fn running_nodes(graph: &GuiGraph) -> Vec<&NodeData> {
//     graph
//         .nodes_ref()
//         .into_iter()
//         .map(|nx| graph.get_node(nx))
//         .filter(|node| matches!(node.status, NodeStatus::Running { .. }))
//         .collect()
// }
//
//
pub fn template_node_size(_template: &ForayNodeTemplate) -> iced::Size {
    default_node_size()

    // match template {
    //     ForayNodeTemplate::RustNode(rn) => match rn {
    //         // RustNode::Linspace(_) => Size::new(dft.width * 2., dft.height),
    //         // RustNode::Plot(_) => dft * 2.,
    //         // RustNode::Plot2D(_) => (dft.width * 2., dft.width * 2.).into(),
    //         // RustNode::VectorField(_) => (dft.width * 2., dft.width * 2.).into(),
    //         _ => dft,
    //     },
    //     ForayNodeTemplate::PyNode(_) => dft,
    // }
}

pub fn node_view<'a>(
    node_instance: &'a ForayNodeInstance,
    id: u32,
    input_data: Dict<String, WireDataContainer<PortData>>,
) -> iced::Element<'a, Message> {
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

    match &node_instance.template {
        ForayNodeTemplate::RustNode(rn) => match rn {
            RustNodeTemplate::Constant(value) => operation("not_implimented"), //constant::view(id, *value),
            // RustNode::Linspace(linspace_config) => linspace_config.view(id),
            // RustNode::Plot(plot) => plot.view(id, input_data),
            // RustNode::Plot2D(plot) => plot.view(id, input_data),
            // RustNode::VectorField(vf) => vf.view(id, input_data),
            RustNodeTemplate::Add => operation("+"),
            RustNodeTemplate::Subtract => operation("−"),
            RustNodeTemplate::Multiply => operation("×"),
            RustNodeTemplate::Divide => operation("÷"),
            RustNodeTemplate::Cos => trig("cos(α)"),
            RustNodeTemplate::Sin => trig("sin(α)"),
            RustNodeTemplate::Sinc => trig("sinc(α)"),

            _ => text(rn.to_string()).into(),
        },
        ForayNodeTemplate::PyNode(py_node) => text(py_node.name.clone()).into(),
    }
}

pub fn config_view<'a>(
    node_instance: &'a ForayNodeInstance,
    id: u32,
    input_data: Dict<String, WireDataContainer<PortData>>,
) -> Option<iced::Element<'a, Message>> {
    match &node_instance.template {
        ForayNodeTemplate::RustNode(rn) => match rn {
            // RustNode::Plot(plot) => plot.config_view(id, input_data),
            // RustNode::Plot2D(plot) => plot.config_view(id, input_data),
            // RustNode::VectorField(plot) => plot.config_view(id, input_data),
            _ => None,
        },
        ForayNodeTemplate::PyNode(pn) => {
            if let Ok(parameters) = pn.parameters() {
                Some(
                    column(parameters.clone().into_iter().map(|(name, widget_type)| {
                        let name_2 = name.clone();
                        let name_3 = name.clone();
                        let message = move |widget_value| {
                            Message::UpdateNodeParameter(id, name.clone(), widget_value)
                        };
                        let message_2 = message.clone();
                        //TODO: make widget type view
                        let widget: Element<Message> = match widget_type {
                            UIParameter::CheckBox(v) => todo!(),
                            UIParameter::NumberField(v) => row![
                                horizontal_space(),
                                row![numeric_input::numeric_input(
                                    v as f32,
                                    numeric_input::PartialUIValue::Complete,
                                    move |new_v, in_progress: PartialUIValue| {
                                        message(PortData::Float(new_v as f64)) //, in_progress))
                                    },
                                )]
                                .width(60.0)
                            ]
                            .align_y(Center)
                            .into(),
                            UIParameter::Slider(start, stop, default_v) => {
                                let current_value =
                                    match node_instance.parameters_values[&name_2.clone()] {
                                        PortData::Float(v) => v,
                                        _ => panic!("slider should be a float"),
                                    };

                                row![
                                    row![
                                        iced_aw::typed_input::TypedInput::new(
                                            "Placeholder",
                                            &current_value
                                        )
                                        .on_input(
                                            move |new_v| Message::UpdateNodeParameter(
                                                id,
                                                name_2.clone(),
                                                PortData::Float(new_v),
                                            )
                                        ) // iced_aw::typed_input(&current_value, move |new_v| {
                                          //     Message::UpdateNodeParameter(
                                          //         id,
                                          //         name.clone(),
                                          //         PortData::Float(new_v as f64),
                                          //     )
                                          // }),
                                          // numeric_input::numeric_input(
                                          // current_value,
                                          // numeric_input::PartialUIValue::Complete,
                                          // move |new_v, pv| {
                                          //     message(PortData::Float(new_v as f64))
                                          //     // UIParameter::Slider(start, stop, new_v as f64))
                                          // },
                                    ]
                                    .width(60.0),
                                    slider(start..=stop, current_value, move |new_v| {
                                        message_2(PortData::Float(
                                            (new_v * PRECISION).round() / PRECISION,
                                        ))
                                        // message(
                                        //     Self::Slider(new_v, PartialUIValue::Complete))
                                    })
                                    .step(1.0 / PRECISION)
                                ]
                                .align_y(Center)
                                .spacing(4.0)
                                .into()
                            }
                        };
                        row![text(name_3.clone()), widget] //widget_type.view(message)]
                            .spacing(8.0)
                            .align_y(Center)
                            .width(Fill)
                            .into()
                    }))
                    .spacing(8.)
                    .width(Fill)
                    .into(),
                )
            } else {
                Some(text("").into())
            }
        }
    }
}
