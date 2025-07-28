use foray_data_model::node::PortType;
use foray_graph::graph::{GraphNode, PortRef, IO};
use iced::{
    alignment::Horizontal::Right,
    border,
    widget::{
        column, container, container::background, mouse_area, row, text, tooltip, vertical_space,
    },
    Alignment::Center,
    Border, Element, Size,
};
use itertools::Itertools;

use crate::{
    app::Message,
    math::Point,
    node_instance::ForayNodeInstance,
    style::{color::Color, theme::AppTheme},
    widget::{custom_button, pin::Pin},
    CODE_FONT,
};

use super::node::{NODE_RADIUS, PORT_RADIUS};

pub fn port_view<'a>(
    node_id: u32,
    node_data: &ForayNodeInstance,
    node_size: Size,
    app_theme: &'a AppTheme,
) -> Vec<Element<'a, Message>> {
    let port_x = |i: usize| i as f32 * (node_size.width / 4.) + NODE_RADIUS * 2.;

    //TODO: Unify in/out port view creation
    let in_port_buttons = node_data
        .inputs()
        .into_iter()
        .enumerate()
        .map(|(i, port)| (Point::new(port_x(i), -PORT_RADIUS), port))
        .map(|(point, port)| {
            let (name, port_type) = port;
            let port_tooltip = port_tooltip(name.clone(), port_type.clone(), app_theme);

            let in_port = PortRef {
                node: node_id,
                name,
                io: IO::In,
            };

            Pin::new(tooltip(
                mouse_area(
                    custom_button::Button::new("")
                        .on_press(Message::PortPress(in_port.clone()))
                        .on_drag(Message::OnMove)
                        .on_right_press(Message::PortDelete(in_port.clone()))
                        .on_release_self(Message::PortRelease)
                        .style(move |_t, s| port_style(port_type.clone(), s, app_theme))
                        .width(PORT_RADIUS * 2.)
                        .height(PORT_RADIUS * 2.),
                )
                .on_enter(Message::PortStartHover(in_port.clone()))
                .on_exit(Message::PortEndHover(in_port.clone())),
                port_tooltip,
                tooltip::Position::Top,
            ))
            .position(point)
            .into()
        });
    let out_port_buttons = node_data
        .outputs()
        .into_iter()
        .enumerate()
        .map(|(i, port)| (Point::new(port_x(i), node_size.height - PORT_RADIUS), port))
        .map(|(point, port)| {
            let (name, port_type) = port;
            let port_tooltip = port_tooltip(name.clone(), port_type.clone(), app_theme);

            let out_port = PortRef {
                node: node_id,
                name,
                io: IO::Out,
            };

            Pin::new(
                mouse_area(tooltip(
                    custom_button::Button::new(vertical_space())
                        .on_press(Message::PortPress(out_port.clone()))
                        .on_drag(Message::OnMove)
                        .on_right_press(Message::PortDelete(out_port.clone()))
                        .on_release_self(Message::PortRelease)
                        .style(move |_t, s| port_style(port_type.clone(), s, app_theme))
                        .width(PORT_RADIUS * 2.)
                        .height(PORT_RADIUS * 2.)
                        .padding(2.0),
                    port_tooltip,
                    tooltip::Position::Bottom,
                ))
                .on_enter(Message::PortStartHover(out_port.clone()))
                .on_exit(Message::PortEndHover(out_port.clone())),
            )
            .position(point)
            .into()
        });
    in_port_buttons.chain(out_port_buttons).collect()
}

fn port_style(
    port_type: PortType,
    s: custom_button::Status,
    app_theme: &AppTheme,
) -> custom_button::Style {
    let color_pair = port_color_pair(&port_type, app_theme);
    let mut style = custom_button::custom(s, color_pair.0.into(), color_pair.1.into());
    style.border.radius = border::radius(100.);
    style
}

/// Get (base, highlight) color pair for port type
fn port_color_pair(port_type: &PortType, app_theme: &AppTheme) -> (Color, Color) {
    match port_type {
        PortType::Object(_) => app_theme.orange.color_pair(),
        PortType::Integer => app_theme.red.color_pair(),
        PortType::Float => app_theme.blue.color_pair(),
        PortType::Boolean => app_theme.cyan.color_pair(),
        PortType::String => app_theme.green.color_pair(),
        PortType::Array(array_port_type, _) => port_color_pair(array_port_type, app_theme),
    }
}

/// Stylized port text
fn port_text(port_type: &PortType) -> String {
    match port_type {
        PortType::Integer => "Integer",
        PortType::Float => "Float",
        PortType::Boolean => "Boolean",
        PortType::String => "String",
        PortType::Array(_port_type, _items) => "Array",
        PortType::Object(_children) => "Object",
    }
    .to_owned()
}
fn port_type_label<'a>(
    label: Element<'a, Message>,
    port_type: &PortType,
    app_theme: &'a AppTheme,
) -> Element<'a, Message> {
    //text::Rich<'a, Message> {
    let color = port_color_pair(port_type, app_theme).0.iced_color();
    container(label)
        .style(move |_| background(color).border(Border::default().rounded(4)))
        .into()
}

/// Display summary of port information
fn port_tooltip(
    port_name: String,
    port_type: PortType,
    app_theme: &'_ AppTheme,
) -> Element<'_, Message> {
    row![
        text(port_name),
        port_tooltip_recurse(port_type, app_theme, true)
    ]
    .align_y(Center)
    .into()
}

const VERTICAL_SPACING: u16 = 2;
const NESTED_PADDING: u16 = 2;
const BORDER_RADIUS: u16 = 4;

fn port_tooltip_recurse(
    //port_name: String,
    port_type: PortType,
    app_theme: &'_ AppTheme,
    even: bool, // Switch between 2 background colors as objects are nested
) -> Element<'_, Message> {
    let nested = |inner| {
        container(column(inner).spacing(VERTICAL_SPACING).align_x(Right)).style(move |_t| {
            background(iced::Color::from(if even {
                app_theme.background.weak_color()
            } else {
                app_theme.background.strong_color()
            }))
            .border(Border::default().rounded(BORDER_RADIUS))
        })
    };

    let port_type_display = match &port_type {
        // Recursive cases
        PortType::Object(fields) => nested(fields.iter().map(|(k, v)| {
            row![
                text(k.clone()),
                port_tooltip_recurse(v.clone(), app_theme, !even)
            ]
            .align_y(Center)
            .into()
        }))
        .padding(NESTED_PADDING)
        .into(),
        PortType::Array(array_type, shape) => {
            let nested_tooltip = port_tooltip_recurse(*array_type.clone(), app_theme, even);
            let shape_str = format!(
                "[{}]",
                shape
                    .iter()
                    .map(|v| match v {
                        Some(v) => v.to_string(),
                        None => ":".to_string(),
                    })
                    .join(",")
            );

            port_type_label(
                row![nested_tooltip, text(shape_str).font(CODE_FONT)]
                    .padding(NESTED_PADDING)
                    .align_y(Center)
                    .into(),
                &port_type,
                app_theme,
            )
        }

        // Base case
        _ => port_type_label(text(port_text(&port_type)).into(), &port_type, app_theme),
    };
    container(port_type_display).padding([0, 4]).into()
}
