use std::f32::consts::PI;
use std::time::Instant;

use crate::app::{App, Message};
use crate::node_instance::{ForayNodeInstance, ForayNodeTemplate, NodeStatus};
use crate::rust_nodes::RustNodeTemplate;
use crate::widget::node_container::NodeContainer;
use crate::StableMap;
use foray_data_model::node::{Dict, PortData};
use foray_data_model::WireDataContainer;

use iced::Alignment::Center;
use iced::Font;
use iced::{
    border,
    widget::{column, *},
    Color, Element,
};

use super::port::port_view;

// TODO: remove hard-coded node sizes, use size specified in node-template,
// or on an individual node level, so that nodes can be dynamically resized
pub const INNER_NODE_WIDTH: f32 = 120.;
pub const INNER_NODE_HEIGHT: f32 = 60.;
pub const PORT_RADIUS: f32 = 8.5;
pub const NODE_RADIUS: f32 = 5.0;
pub const NODE_BORDER_WIDTH: f32 = 2.0;
pub const OUTER_NODE_WIDTH: f32 = INNER_NODE_WIDTH + NODE_BORDER_WIDTH;
pub const OUTER_NODE_HEIGHT: f32 = INNER_NODE_HEIGHT + NODE_BORDER_WIDTH;

pub fn default_node_size() -> iced::Size {
    iced::Size::new(OUTER_NODE_WIDTH, OUTER_NODE_HEIGHT)
}

impl App {
    pub fn node_content(&'_ self, id: u32) -> Element<'_, Message, Theme, Renderer> {
        let node = self.network.graph.get_node(id);
        let is_selected = self.network.selected_shapes.contains(&id);

        let node_style = move |node: &ForayNodeInstance, t: &Theme| {
            let color = match &node.status {
                NodeStatus::Idle | NodeStatus::Running { .. } => match is_selected {
                    true => t.extended_palette().primary.strong.color,
                    false => t.extended_palette().secondary.strong.color,
                },
                NodeStatus::Error(_py_node_error) => t.extended_palette().danger.strong.color,
            };
            let run_time = match &node.status {
                NodeStatus::Running { start: start_inst } => {
                    (Instant::now() - *start_inst).as_secs_f32()
                }
                _ => 0.0,
            };

            let pulse_freq = 1.0;
            let pulse = color.scale_alpha(
                ((run_time  // t
                        * pulse_freq * 2.0 * PI // f 
                    ).cos() // start at 1.0
                    + 1.0 // shift range 0.0-2.0
                ) * 0.5 // scale range 0.0-1.0
                    * 0.75  // scale range 0.0-0.75
                    + 0.25, // shift range 0.25-1.0
            );

            container::transparent(t)
                .border(
                    border::rounded(NODE_RADIUS)
                        .color(pulse)
                        .width(NODE_BORDER_WIDTH),
                )
                .background(iced::Color::from(self.app_theme.background.base_color))
        };

        //// Node
        let input_data = self.network.graph.get_input_data(&id);
        let node_size = template_node_size(&node.template);

        //// Ports
        let show_port_tooltips = !matches!(self.action, crate::app::Action::AddingNode);
        let port_buttons = port_view(id, node, node_size, &self.app_theme, show_port_tooltips);

        let node_primary = container(node_view(node, id, input_data))
            .style(move |theme| node_style(node, theme))
            .center_x(node_size.width)
            .center_y(node_size.height);

        //// Secondary Node view (data vis)
        let data_display_size = default_node_size().height * 1.0;

        let node_secondary = match &node.visualization.image_handle {
            Some(h) => container(
                image(h)
                    .filter_method(image::FilterMethod::Nearest)
                    .height(data_display_size)
                    .width(data_display_size),
            ),
            None => container(""),
        };
        //let node_secondary =
        let node: iced::Element<Message> =
            row![node_primary, node_secondary].align_y(Center).into();

        let content: Element<Message, Theme, Renderer> = NodeContainer::new(
            if self.debug {
                node.explain(Color::from_rgba(0.7, 0.7, 0.8, 0.2))
            } else {
                node
            },
            port_buttons,
        )
        //.width(node_size.width)
        //.height(node_size.height)
        .into();
        content
    }
}

pub fn format_node_debug_output<'a>(
    node: &ForayNodeInstance,
    data: &StableMap<String, Option<&WireDataContainer<PortData>>>,
) -> iced::Element<'a, Message> {
    //TODO: Clean this up by iterating straight to text elements?
    let node_output = data.iter().map(|(port_name, d)| {
        (
            port_name.to_string(),
            d.map(|d| format!("{:?}", d.read().unwrap()))
                .unwrap_or("n/a".to_string()),
        )
    });

    container(column![
        text(format!("{node:#?}")).size(12.),
        column(node_output.map(|(lbl, val)| {
            row![text(lbl).size(12.), text(val).size(12.)]
                .spacing(5.0)
                .into()
        }))
    ])
    .into()
}

pub fn node_view<'a>(
    node_instance: &'a ForayNodeInstance,
    _id: u32,
    _input_data: Dict<String, WireDataContainer<PortData>>,
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
            RustNodeTemplate::Constant(_value) => operation("not_implimented"), //constant::view(id, *value),
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
