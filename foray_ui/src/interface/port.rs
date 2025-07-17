use foray_data_model::node::PortType;
use foray_graph::{
    graph::{GraphNode, PortRef, IO},
    node_instance::ForayNodeInstance,
};
use iced::{
    border, color,
    widget::{
        column, container, container::background, mouse_area, rich_text, row, span, text, tooltip,
        vertical_space, Row,
    },
    Alignment::Center,
    Border, Element, Size,
};

use crate::{
    app::Message,
    math::Point,
    style::theme::AppTheme,
    widget::{custom_button, pin::Pin},
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
    let mut style = custom_button::custom(s, color_pair.0, color_pair.1);
    style.border.radius = border::radius(100.);
    style
}

/// Get (base, highlight) color pair for port type
fn port_color_pair(port_type: &PortType, app_theme: &AppTheme) -> (iced::Color, iced::Color) {
    match port_type {
        //TODO: Put these colors into AppTheme
        // PortType::Integer => (color!(175, 48, 41), color!(209, 77, 65)), //red
        // PortType::Real => (
        //     app_theme.primary.base_color.into(),
        //     app_theme.primary.weak_color().into(),
        // ),
        // PortType::Complex => (color!(102, 128, 11), color!(135, 154, 57)), //green
        // PortType::ArrayInteger => (color!(175, 48, 41), color!(209, 77, 65)), //red
        // PortType::ArrayReal => (color!(32, 94, 166), color!(67, 133, 190)), //blue
        // PortType::ArrayComplex => (color!(36, 131, 123), color!(58, 169, 159)), //cyan
        // PortType::Dynamic => (color!(175, 125, 41), color!(209, 150, 65)), //orange
        PortType::Object(_) => (color!(200, 160, 41), color!(229, 180, 65)), //yellow
        _ => (color!(175, 48, 41), color!(209, 77, 65)),                     //red
    }
}

/// Stylized port text
fn port_text<'a>(
    port_name: String,
    port_type: &PortType,
    app_theme: &AppTheme,
) -> Row<'a, Message> {
    row![
        text(port_name),
        rich_text([span("todo_port_name".to_string()) //port_type.to_string())
            .background(port_color_pair(port_type, app_theme).0)
            .border(Border::default().rounded(4))
            .padding([0, 2])])
    ]
}

/// Display summary of port information
fn port_tooltip(
    port_name: String,
    port_type: PortType,
    app_theme: &'_ AppTheme,
) -> Element<'_, Message> {
    port_tooltip_recurse(port_name, port_type, app_theme, true)
}

fn port_tooltip_recurse(
    port_name: String,
    port_type: PortType,
    app_theme: &'_ AppTheme,
    even: bool, // Switch between 2 background colors as objects are nested
) -> Element<'_, Message> {
    let port_type_display = match port_type {
        // Recursive case
        PortType::Object(fields) => row![
            text(port_name),
            container(
                column(
                    fields
                        .into_iter()
                        .map(|(k, v)| port_tooltip_recurse(k, v, app_theme, !even))
                )
                .spacing(4)
            )
            .padding(4)
            .style(move |_t| background(iced::Color::from(if even {
                app_theme.background.weak_color()
            } else {
                app_theme.background.strong_color()
            }))
            .border(Border::default().rounded(4)))
        ]
        .spacing(4)
        .align_y(Center),
        // Base case
        _ => port_text(port_name.clone(), &port_type, app_theme).spacing(8),
    };
    container(port_type_display).padding([0, 4]).into()
}
