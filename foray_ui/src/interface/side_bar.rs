use crate::interface::status::node_status_widget;
use crate::interface::SEPERATOR;
use crate::node_instance::{ForayNodeInstance, ForayNodeTemplate};
use crate::rust_nodes::RustNodeTemplate;
use crate::style::button::{primary_icon, secondary_icon};
use crate::style::icon::icon;
use crate::workspace::{Workspace, WorkspaceMessage};
use foray_data_model::node::{Dict, PortData, UIParameter};
use foray_data_model::WireDataContainer;
use iced::widget::container::background;
use iced::*;
use widget::{column, *};

use super::numeric_input::{self, PartialUIValue};

const PRECISION: f64 = 100.0;
/// Create the sidebar view
pub fn side_bar(app: &Workspace) -> Element<'_, WorkspaceMessage> {
    fn file_button<'a>(
        lbl: impl Into<String>,
        message: WorkspaceMessage,
    ) -> Button<'a, WorkspaceMessage> {
        button(icon(lbl.into()))
            .on_press(message)
            .style(primary_icon)
            .padding(0.0)
    }
    fn undo_button<'a>(
        lbl: impl Into<String>,
        enabled: bool,
        message: WorkspaceMessage,
    ) -> Button<'a, WorkspaceMessage> {
        button(icon(lbl.into()))
            .on_press_maybe(if enabled { Some(message) } else { None })
            .padding(0.0)
            .style(secondary_icon)
    }

    //''
    //''
    //''
    //''
    //''
    let file_commands = row![
        file_button('󰝒', WorkspaceMessage::New),
        file_button('󰝰', WorkspaceMessage::StartLoadNetwork),
        file_button('󰆓', WorkspaceMessage::StartSaveNetwork),
        //file_button('󰃤', WorkspaceMessage::ToggleDebug),
        //file_button('󰏘', WorkspaceMessage::TogglePaletteUI),
    ]
    .spacing(3.0);

    let undo = undo_button(
        '',
        //debug_format(&app.debug, '', app.network.undo_stack.len()),
        !app.network.undo_stack.is_empty(),
        WorkspaceMessage::Undo,
    );
    let redo = undo_button(
        //debug_format(&app.debug, '', app.network.redo_stack.len()),
        '',
        !app.network.redo_stack.is_empty(),
        WorkspaceMessage::Redo,
    );
    let action_commands = row![horizontal_space(), undo, redo].spacing(4.0);

    //// Config
    let config: Element<WorkspaceMessage> =
        if let Some(selected_id) = app.network.selected_shapes.iter().next() {
            let node = app.network.graph.get_node(*selected_id);
            let input_data = app.network.graph.get_input_data(selected_id);
            //let out_port_display: Element<WorkspaceMessage> = if app.debug {
            //    column![format_node_debug_output(
            //        node,
            //        &app.network.graph.get_output_data(selected_id)
            //    )]
            //    .into()
            //} else {
            //    text("").into()
            //};
            column![
                container(text(node.template.name().clone()).size(20.)).center_x(Fill),
                horizontal_rule(0),
                node_status_widget(&node.status),
                vertical_space().height(10.),
                config_view(node, *selected_id, input_data).unwrap_or(text("...").into()),
                // node.config_view(*selected_id, input_data)
                //     .unwrap_or(text("...").into()),
                vertical_space(),
                //scrollable(out_port_display),
                row![button(text("delete node"))
                    .style(button::danger)
                    .padding([1, 4])
                    .on_press(WorkspaceMessage::DeleteSelectedNodes)]
            ]
            .align_x(Center)
            .height(Fill)
            .spacing(5.)
            .padding([10., 5.])
            .into()
        } else {
            text("").into()
        };
    container(
        column![
            row![
                //// File
                file_commands.align_y(Alignment::Center),
                horizontal_space(),
                //// Actions
                action_commands.align_y(Alignment::Center),
            ]
            .padding([2., 4.]),
            horizontal_rule(SEPERATOR),
            //// Config
            config
        ]
        .height(Fill)
        .width(200.),
    )
    .style(|theme| background(theme.palette().background))
    .into()
}

pub fn config_view<'a>(
    node_instance: &'a ForayNodeInstance,
    id: u32,
    _input_data: Dict<String, WireDataContainer<PortData>>,
) -> Option<iced::Element<'a, WorkspaceMessage>> {
    match &node_instance.template {
        ForayNodeTemplate::RustNode(RustNodeTemplate::Display) => {
            Some(node_instance.visualization.parameters.view(id))
        }
        // TODO: data is awkwardly stored in two locations, defaults are in the widget_type,
        // current values are stored in node_instance.parameter_values. these values are less
        // structured (they could be any PortData type)
        //
        ForayNodeTemplate::PyNode(pn) => {
            if let Ok(parameters) = pn.parameters() {
                Some(
                    column(parameters.clone().into_iter().map(|(name, widget_type)| {
                        let name_2 = name.clone();
                        let name_3 = name.clone();
                        let message = move |widget_value| {
                            WorkspaceMessage::UpdateNodeParameter(id, name.clone(), widget_value)
                        };
                        let message_2 = message.clone();
                        //TODO: make widget type view
                        let widget: Element<WorkspaceMessage> = match widget_type {
                            UIParameter::CheckBox(_initial_v) => {
                                row![
                                    horizontal_space(),
                                    toggler(
                                        match node_instance
                                            .parameters_values
                                            .get(&name_2)
                                            .expect("parameter should exist")
                                        {
                                            PortData::Boolean(v) => *v,
                                            _ => panic!("unexpected port type"),
                                        }
                                    )
                                    .on_toggle(
                                        move |v| message(PortData::Boolean(v)) //, in_progress))
                                    )
                                    .width(60.0)
                                ]
                                .align_y(Center)
                                .into()
                            }
                            UIParameter::NumberField(_v) => {
                                let current_value =
                                    match node_instance.parameters_values[&name_2.clone()] {
                                        PortData::Float(v) => v,
                                        _ => panic!("slider should be a float"),
                                    };
                                row![
                                    horizontal_space(),
                                    row![numeric_input::numeric_input(
                                        current_value as f32,
                                        numeric_input::PartialUIValue::Complete,
                                        move |new_v, _in_progress: PartialUIValue| {
                                            message(PortData::Float(new_v as f64))
                                            //, in_progress))
                                        },
                                    )]
                                    .width(60.0)
                                ]
                                .align_y(Center)
                                .into()
                            }
                            UIParameter::Slider(start, stop, _default_v) => {
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
                                            move |new_v| WorkspaceMessage::UpdateNodeParameter(
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
