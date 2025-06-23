use std::f32::consts::PI;
use std::time::Instant;

use crate::app::{App, Message};
use crate::gui_node::{GUINode, PortDataContainer};
use crate::nodes::status::NodeStatus;
use crate::nodes::NodeData;
use crate::widget::node_container::NodeContainer;
use crate::StableMap;
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

        let node_style = move |node: &NodeData, t: &Theme| {
            let color = match &node.status {
                NodeStatus::Idle | NodeStatus::Running(_) => match is_selected {
                    true => t.extended_palette().primary.strong.color,
                    false => t.extended_palette().secondary.strong.color,
                },
                NodeStatus::Error(_node_error) => match is_selected {
                    true => t.extended_palette().danger.base.color,
                    false => t.extended_palette().danger.weak.color,
                },
            };
            let run_time = match &node.status {
                NodeStatus::Running(start_inst) => (Instant::now() - *start_inst).as_secs_f32(),
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
        let node_size = node.template.node_size();
        let node_view = node.template.view(id, input_data);

        //// Ports
        let port_buttons = port_view(id, node, node_size, &self.app_theme);

        let node_inner: Element<Message, Theme, Renderer> = container(node_view)
            .style(move |theme| node_style(node, theme))
            .center_x(node_size.width)
            .center_y(node_size.height)
            .into();

        let content: Element<Message, Theme, Renderer> = NodeContainer::new(
            if self.debug {
                node_inner.explain(Color::from_rgba(0.7, 0.7, 0.8, 0.2))
            } else {
                node_inner
            },
            port_buttons,
        )
        .width(node_size.width)
        .height(node_size.height)
        .into();
        content
    }
}

pub fn format_node_output<'a>(
    node: &NodeData,
    data: &StableMap<String, Option<&PortDataContainer>>,
) -> Element<'a, Message> {
    //TODO: Clean this up by iterating straight to text elements?
    let node_output = data.iter().map(|(port_name, d)| {
        (
            port_name.to_string(),
            d.map(|d| format!("{}", d.read().unwrap()))
                .unwrap_or("n/a".to_string()),
        )
    });

    container(column![
        text(format!("{:#?}", node)).size(12.),
        column(node_output.map(|(lbl, val)| {
            row![text(lbl).size(12.), text(val).size(12.)]
                .spacing(5.0)
                .into()
        }))
    ])
    .into()
}
