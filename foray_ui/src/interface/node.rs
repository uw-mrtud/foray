use std::f32::consts::PI;
use std::time::Instant;

use crate::interface::port::port_color_pair;
use crate::node_instance::{ForayNodeInstance, ForayNodeTemplate, NodeStatus};
use crate::rust_nodes::RustNodeTemplate;
use crate::style::theme::AppTheme;
use crate::widget::node_container::NodeContainer;
use crate::workspace::{Action, Workspace, WorkspaceMessage};
use crate::StableMap;
use foray_data_model::node::{Dict, PortData, PortType};
use foray_data_model::WireDataContainer;
use foray_graph::graph::GraphNode;

use iced::font::Family;
use iced::mouse::Cursor;
use iced::widget::canvas::{stroke, Path, Text};
use iced::Alignment::Center;
use iced::{
    border,
    widget::{column, *},
    Element,
};
use iced::{Font, Rectangle};

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
pub const NODE_TEXT_SIZE: f32 = 24.0;

pub fn default_node_size() -> iced::Size {
    iced::Size::new(OUTER_NODE_WIDTH, OUTER_NODE_HEIGHT)
}

impl ForayNodeInstance {
    pub fn node_bounding_rect(&self) -> Rectangle {
        let estimated_text_width = NODE_TEXT_SIZE * self.template.name().len() as f32 * 0.75;
        let node_padding = 8.0;

        Rectangle::new(
            (0.0, 0.0).into(),
            (
                estimated_text_width + node_padding,
                NODE_TEXT_SIZE + node_padding,
            )
                .into(),
        )
    }
    pub fn port_positions(
        &self,
    ) -> (
        Vec<(Rectangle, String, PortType)>,
        Vec<(Rectangle, String, PortType)>,
    ) {
        fn build_port_list(
            ports: Dict<String, PortType>,
            position_ports: impl Fn(usize) -> Rectangle,
        ) -> Vec<(Rectangle, String, PortType)> {
            ports
                .iter()
                .enumerate()
                .map(|(i, (port_name, port_type))| {
                    (position_ports(i), port_name.clone(), port_type.clone())
                })
                .collect()
        }
        (
            build_port_list(self.inputs(), |i| self.input_port_bounding(i)),
            build_port_list(self.outputs(), |i| self.output_port_bounding(i)),
        )
    }
    pub fn input_port_bounding(&self, port_index: usize) -> Rectangle {
        let start_x = NODE_RADIUS;
        let port_width = 12.0;
        let port_height = 12.0;
        let x_spacing = 4.0;
        Rectangle::new(
            (
                start_x
                    + port_index as f32 * (port_width)
                    + port_index.saturating_sub(0) as f32 * x_spacing,
                -port_height,
            )
                .into(),
            (port_width, port_height).into(),
        )
    }

    pub fn output_port_bounding(&self, port_index: usize) -> Rectangle {
        let input = self.input_port_bounding(port_index);

        Rectangle::new(
            (input.x, self.node_bounding_rect().height).into(),
            input.size(),
        )
    }
}

pub fn draw_node(
    // Draw directly into frame
    frame: &mut iced::widget::canvas::Frame,
    cursor: Cursor,
    // only needed for stroke width, all other scaleing is already accounted for
    scale: f32,
    node: &ForayNodeInstance,
    is_selected: bool,
    app_theme: &AppTheme,
) {
    let node_bounding = node.node_bounding_rect();
    //// Name
    let text = Text {
        content: node.template.name(),
        position: node_bounding.center(),
        color: app_theme.text.strong_color().into(),
        size: iced::Pixels(NODE_TEXT_SIZE),
        font: Font {
            family: Family::Name("Courier New"),
            ..Font::default()
        },
        horizontal_alignment: iced::alignment::Horizontal::Center,
        vertical_alignment: iced::alignment::Vertical::Center,
        shaping: text::Shaping::Basic,
        ..Default::default()
    };

    //// Draw text on top of everything, much faster but will cause overlap issues
    frame.fill_text(text);

    //// Draw text with proper layers, but is much slower. Won't scale well with lots of nodes
    //// This method also can let us calculate the text width more accurately
    // let mut text_bounding: Option<iced::Rectangle<f32>> = None; //iced::Rectangle::<f32>::default();
    // text.draw_with(|path, _color| {
    //     frame.fill(&path, app_theme.text.strong_color().iced_color());
    //     // akwardly get bounding box in closure, since there is no other way to acess text's path
    //     let char_bounding = path_bounding_rect(path);
    //     text_bounding = Some(
    //         text_bounding
    //             .map(|r| r.union(&char_bounding))
    //             .unwrap_or(char_bounding),
    //     );
    // });
    // let text_bounding = text_bounding.unwrap_or_default();

    //// Border
    let node_border = Path::rounded_rectangle(
        // text_bounding.position(),
        // text_bounding.size(),
        node_bounding.position(),
        node_bounding.size(),
        NODE_RADIUS.into(),
    );

    let node_border_color = match is_selected {
        true => app_theme.primary.strong_color().into(),
        false => app_theme.text.strong_color().into(),
    };
    let stroke = stroke::Stroke::default()
        .with_color(node_border_color)
        .with_width(2.0 * scale);
    frame.stroke(&node_border, stroke);
    frame.fill(&node_border, app_theme.background.base_color.iced_color());

    //// Image
    if let Some(image_handle) = &node.visualization.image_handle {
        let image_size = 60.0;
        let image_bounds = Rectangle::new(
            // (8.0, node_bounding.height).into(),
            (node_bounding.width + 1.0, 0.0).into(),
            (image_size, image_size).into(),
        );
        frame.draw_image(image_bounds, image_handle);
    };

    //// Ports
    let (input_ports, output_ports) = node.port_positions();

    input_ports.iter().for_each(|(rect, _name, port_type)| {
        let (base, highlight) = port_color_pair(port_type, app_theme);

        let fill_color = match cursor.is_over(*rect) {
            true => highlight,
            false => base,
        };

        frame.stroke_rectangle(
            rect.position(),
            rect.size(),
            stroke.with_color(highlight.into()),
        );
        frame.fill_rectangle(rect.position(), rect.size(), fill_color.iced_color());
    });

    output_ports.iter().for_each(|(rect, _name, port_type)| {
        let (base, highlight) = port_color_pair(port_type, app_theme);
        let fill_color = match cursor.is_over(*rect) {
            true => highlight,
            false => base,
        };
        frame.stroke_rectangle(
            rect.position(),
            rect.size(),
            stroke.with_color(highlight.into()),
        );
        frame.fill_rectangle(rect.position(), rect.size(), fill_color.iced_color());
    });
}

// fn _path_bounding_rect(path: Path) -> iced::Rectangle {
//     let points_iter = path
//         .raw()
//         .iter()
//         .map(|o| match o {
//             canvas::path::lyon_path::Event::Begin { at } => at,
//             canvas::path::lyon_path::Event::Line { to, .. } => to,
//             canvas::path::lyon_path::Event::Quadratic { to, .. } => to,
//             canvas::path::lyon_path::Event::Cubic { to, .. } => to,
//             canvas::path::lyon_path::Event::End { last, .. } => last,
//         })
//         .map(|p| (p.x, p.y));
//
//     let (x_min, x_max) = points_iter
//         .clone()
//         .map(|p| p.0)
//         .minmax_by(|x1, x2| x1.total_cmp(x2))
//         .into_option()
//         .unwrap_or_default();
//     let (y_min, y_max) = points_iter
//         .map(|p| p.1)
//         .minmax_by(|y1, y2| y1.total_cmp(y2))
//         .into_option()
//         .unwrap_or_default();
//
//     iced::Rectangle::new((x_min, y_min).into(), (x_max - x_min, y_max - y_min).into())
// }

impl Workspace {
    pub fn node_content<'a>(
        &'a self,
        id: u32,
        app_theme: &'a AppTheme,
    ) -> Element<'a, WorkspaceMessage, Theme, Renderer> {
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
                .background(iced::Color::from(app_theme.background.base_color))
        };

        //// Node
        let input_data = self.network.graph.get_input_data(&id);
        let output_data = self.network.graph.get_output_data(&id);
        let node_size = template_node_size(&node.template);

        //// Ports
        let show_port_tooltips = !matches!(self.action, Action::AddingNode);
        let port_buttons = port_view(
            id,
            node,
            &input_data,
            &output_data,
            node_size,
            app_theme,
            show_port_tooltips,
        );

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
                    // .content_fit(iced::ContentFit::ScaleDown)
                    .height(data_display_size)
                    .width(data_display_size),
            ),
            None => container(""),
        };
        //let node_secondary =
        let node: iced::Element<WorkspaceMessage> =
            row![node_primary, node_secondary].align_y(Center).into();

        let content: Element<WorkspaceMessage, Theme, Renderer> = NodeContainer::new(
            //if self.debug {
            //    node.explain(Color::from_rgba(0.7, 0.7, 0.8, 0.2))
            //} else {
            node,
            //},
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
) -> iced::Element<'a, WorkspaceMessage> {
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
) -> iced::Element<'a, WorkspaceMessage> {
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
