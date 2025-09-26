use foray_data_model::node::PortType;
use iced::{
    alignment::Horizontal::Right,
    widget::{column, container, container::background, row, text},
    Alignment::Center,
    Border, Element,
};
use itertools::Itertools;

use crate::{
    style::{color::Color, theme::AppTheme},
    CODE_FONT,
};

// fn get_filled_type(
//     name: &String,
//     input_data: &BTreeMap<String, Arc<RwLock<PortData>>>,
// ) -> Option<PortType> {
//     if let Some(port_data) = input_data.get(name) {
//         Some((&*port_data.read().unwrap()).into())
//     } else {
//         None
//     }
// }
//
// fn port_style(
//     port_type: PortType,
//     filled_type: Option<PortType>,
//     s: custom_button::Status,
//     app_theme: &AppTheme,
// ) -> custom_button::Style {
//     let color_pair = port_color_pair(&port_type, app_theme);
//     let filled_color = match filled_type {
//         Some(pt) => port_color_pair(&pt, app_theme).0,
//         None => app_theme.background.base_color.into(),
//     };
//
//     let mut style = custom_button::custom(
//         s,
//         color_pair.0.into(),
//         color_pair.1.into(),
//         filled_color.into(),
//     );
//     style.border.radius = border::radius(100.);
//     style
// }

/// Get (base, highlight) color pair for port type
pub fn port_color_pair(port_type: &PortType, app_theme: &AppTheme) -> (Color, Color) {
    match port_type {
        PortType::Object(_) => app_theme.orange.color_pair(),
        PortType::Integer => app_theme.red.color_pair(),
        PortType::Float => app_theme.blue.color_pair(),
        PortType::Complex => app_theme.orange.color_pair(),
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
        PortType::Complex => "Complex",
        PortType::Boolean => "Boolean",
        PortType::String => "String",
        PortType::Array(_port_type, _items) => "Array",
        PortType::Object(_children) => "Object",
    }
    .to_owned()
}
fn port_type_label<'a, M: 'a>(
    label: Element<'a, M>,
    port_type: &PortType,
    app_theme: &'a AppTheme,
) -> Element<'a, M> {
    //text::Rich<'a, Message> {
    let color = port_color_pair(port_type, app_theme).0.iced_color();
    container(label)
        .style(move |_| background(color).border(Border::default().rounded(4)))
        .into()
}

/// Display summary of port information
pub fn port_tooltip<'a, M: 'a>(
    port_name: String,
    port_type: PortType,
    app_theme: &'a AppTheme,
) -> Element<'a, M> {
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

fn port_tooltip_recurse<'a, M: 'a>(
    //port_name: String,
    port_type: PortType,
    app_theme: &'a AppTheme,
    even: bool, // Switch between 2 background colors as objects are nested
) -> Element<'a, M> {
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
