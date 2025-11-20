use std::f32::consts::TAU;
use std::time::Instant;

use crate::interface::port::port_color_pair;
use crate::node_instance::visualiztion::{NDimVis, SeriesVis, Visualization};
use crate::node_instance::{ForayNodeInstance, ForayNodeTemplate};
use crate::rust_nodes::RustNodeTemplate;
use crate::style::theme::AppTheme;
use crate::workspace::{Action, WorkspaceMessage};
use crate::StableMap;
use foray_data_model::node::{Dict, PortData, PortType};
use foray_data_model::WireDataContainer;
use foray_graph::graph::{GraphNode, PortRef, IO};

use iced::font::Family;
use iced::mouse::Cursor;
use iced::widget::canvas::{stroke, Path, Stroke, Text};
use iced::widget::{column, *};
use iced::{Font, Rectangle};

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
        let node_padding = 8.0;
        match self.template {
            ForayNodeTemplate::RustNode(
                RustNodeTemplate::Display | RustNodeTemplate::DisplaySeries,
            ) => match &self.visualization {
                Some(vis) => match vis {
                    Visualization::NDimVis(ndim_vis) => ndim_vis.parameters.image_bounds(200.0),
                    Visualization::Series(_series_vis) => {
                        Rectangle::new((0.0, 0.0).into(), (300.0, 300.0).into())
                    }
                },
                None => Rectangle::new(
                    (0.0, 0.0).into(),
                    (200.0, NODE_TEXT_SIZE + node_padding).into(),
                ),
            },
            _ => {
                let estimated_text_width =
                    NODE_TEXT_SIZE * self.template.name().len() as f32 * 0.75;

                Rectangle::new(
                    (0.0, 0.0).into(),
                    (
                        estimated_text_width + node_padding,
                        NODE_TEXT_SIZE + node_padding,
                    )
                        .into(),
                )
            }
        }
    }
    pub fn port_positions(
        &self,
        node_id: u32,
    ) -> (
        Vec<(Rectangle, PortRef, PortType)>,
        Vec<(Rectangle, PortRef, PortType)>,
    ) {
        fn build_port_list(
            ports: Dict<String, PortType>,
            position_ports: impl Fn(usize) -> Rectangle,
            node_id: u32,
            io: IO,
        ) -> Vec<(Rectangle, PortRef, PortType)> {
            ports
                .into_iter()
                .enumerate()
                .map(|(i, (port_name, port_type))| {
                    (
                        position_ports(i),
                        PortRef {
                            node: node_id,
                            name: port_name,
                            io,
                        },
                        port_type,
                    )
                })
                .collect()
        }
        (
            build_port_list(
                self.inputs(),
                |i| self.input_port_bounding(i),
                node_id,
                IO::In,
            ),
            build_port_list(
                self.outputs(),
                |i| self.output_port_bounding(i),
                node_id,
                IO::Out,
            ),
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
    // only needed for stroke width, all other scaling is already accounted for
    scale: f32,
    node: &ForayNodeInstance,
    node_id: u32,
    action: Action,
    is_selected: bool,
    app_theme: &AppTheme,
) {
    let node_border_color = match &node.status {
        crate::node_instance::NodeStatus::Idle { .. } => match is_selected {
            true => app_theme.primary.strong_color().into(),
            false => app_theme.text.weak_color().into(),
        },
        crate::node_instance::NodeStatus::Running { start } => {
            let duration = (Instant::now() - *start).as_secs_f32();
            let base_color = match is_selected {
                true => app_theme.primary.strong_color(),
                false => app_theme.text.weak_color(),
            };
            // Osciallation parameters
            let pulse_per_second = 0.5;
            let min_alpha = 0.1;
            // cos wave starting at 1.0, varies from 1.0 to min_alpha
            let alpha = ((duration * pulse_per_second * TAU).cos() + 1.0 + min_alpha)
                / ((1.0 + min_alpha) * 2.0);
            base_color.iced_color().scale_alpha(alpha)
        }
        crate::node_instance::NodeStatus::Error(_foray_node_errors) => match is_selected {
            true => app_theme.red.strong_color().into(),
            false => app_theme.red.weak_color().into(),
        },
    };
    //    true => app_theme.primary.strong_color().into(),
    let stroke = stroke::Stroke::default()
        .with_color(node_border_color)
        .with_width(2.0 * scale);

    let node_bounding = node.node_bounding_rect();
    let node_background = app_theme.background.base_color.iced_color();

    match node.template {
        // Draw Display Node
        ForayNodeTemplate::RustNode(
            RustNodeTemplate::Display | RustNodeTemplate::DisplaySeries,
        ) => {
            let img_padding = 1.0;
            let (image_width, image_height) = (
                node_bounding.size().width - img_padding,
                node_bounding.size().height - img_padding,
            );
            let image_bounds = Rectangle::new(
                (img_padding / 2.0, img_padding / 2.0).into(),
                (image_width, image_height).into(),
            );

            draw_node_border(frame, node_bounding, 1.0, stroke, node_background);
            match &node.visualization {
                Some(vis) => match vis {
                    Visualization::NDimVis(ndim_vis) => {
                        draw_node_image(frame, ndim_vis, image_bounds);
                    }
                    Visualization::Series(series_vis) => {
                        draw_node_svg(frame, series_vis, image_bounds);
                    }
                },
                None => {}
            };
            draw_node_ports(frame, app_theme, node, node_id, action, cursor, stroke);
        }
        // Draw Default Node
        _ => {
            draw_node_text(frame, app_theme, node_bounding, node.template.name());
            draw_node_border(frame, node_bounding, NODE_RADIUS, stroke, node_background);
            match &node.visualization {
                Some(vis) => match vis {
                    Visualization::NDimVis(ndim_vis) => {
                        let image_size = ndim_vis.parameters.image_bounds(60.0).size();
                        let image_bounds =
                            Rectangle::new((node_bounding.width, 0.0).into(), image_size);
                        draw_node_image(frame, ndim_vis, image_bounds);
                    }
                    Visualization::Series(_series_vis) => {}
                },
                None => {}
            };
            draw_node_ports(frame, app_theme, node, node_id, action, cursor, stroke);
        }
    };
}

pub fn draw_node_text(
    // Draw directly into frame
    frame: &mut iced::widget::canvas::Frame,
    app_theme: &AppTheme,
    node_bounding: Rectangle,
    name: String,
) {
    let text = Text {
        content: name,
        position: node_bounding.center(),
        color: app_theme.text.strong_color().into(),
        size: iced::Pixels(NODE_TEXT_SIZE),
        font: Font {
            family: Family::Name("Courier New"),
            ..Font::default()
        },
        align_x: text::Alignment::Center,
        align_y: iced::alignment::Vertical::Center,
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
}
pub fn draw_node_border(
    // Draw directly into frame
    frame: &mut iced::widget::canvas::Frame,
    node_bounding: Rectangle,
    node_radius: f32,
    stroke: Stroke,
    fill: iced::Color,
) {
    let node_border = Path::rounded_rectangle(
        // text_bounding.position(),
        // text_bounding.size(),
        node_bounding.position(),
        node_bounding.size(),
        node_radius.into(),
    );

    frame.stroke(&node_border, stroke);
    frame.fill(&node_border, fill);
}
pub fn draw_node_ports(
    // Draw directly into frame
    frame: &mut iced::widget::canvas::Frame,
    app_theme: &AppTheme,
    node: &ForayNodeInstance,
    node_id: u32,
    action: Action,
    cursor: Cursor,
    stroke: Stroke,
) {
    let (input_ports, output_ports) = node.port_positions(node_id);

    input_ports
        .iter()
        .chain(output_ports.iter())
        .for_each(|(rect, port_ref, port_type)| {
            let (base, highlight) = port_color_pair(port_type, app_theme);

            let fill_color = match cursor.is_over(*rect) {
                true => highlight,
                false => base,
            };
            let fill_alpha = match &action {
                Action::CreatingInputWire(creation_port)
                | Action::CreatingOutputWire(creation_port) => {
                    if port_ref == creation_port {
                        0.5
                    } else {
                        1.0
                    }
                }
                _ => 1.0,
            };

            frame.stroke_rectangle(
                rect.position(),
                rect.size(),
                stroke.with_color(highlight.into()),
            );
            frame.fill_rectangle(
                rect.position(),
                rect.size(),
                fill_color.iced_color().scale_alpha(fill_alpha),
            );
        });
}

pub fn draw_node_image(
    // Draw directly into frame
    frame: &mut iced::widget::canvas::Frame,
    ndim_vis: &NDimVis,
    image_bounds: Rectangle,
) {
    if let Some(image_handle) = &ndim_vis.image_handle {
        let img = iced::advanced::image::Image::from(image_handle)
            .filter_method(image::FilterMethod::Nearest)
            .snap(true);
        frame.draw_image(image_bounds, img);
    };
}
pub fn draw_node_svg(
    // Draw directly into frame
    frame: &mut iced::widget::canvas::Frame,
    series_vis: &SeriesVis,
    svg_bounds: Rectangle,
) {
    if let Some(svg) = &series_vis.svg {
        // dbg!(svg);
        //TODO: Do I have to clone?
        frame.draw_svg(svg_bounds, &svg.handle);
    };
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

//         let is_selected = self.network.selected_shapes.contains(&id);
//
//         let node_style = move |node: &ForayNodeInstance, t: &Theme| {
//             let color = match &node.status {
//                 NodeStatus::Idle | NodeStatus::Running { .. } => match is_selected {
//                     true => t.extended_palette().primary.strong.color,
//                     false => t.extended_palette().secondary.strong.color,
//                 },
//                 NodeStatus::Error(_py_node_error) => t.extended_palette().danger.strong.color,
//             };
//             let run_time = match &node.status {
//                 NodeStatus::Running { start: start_inst } => {
//                     (Instant::now() - *start_inst).as_secs_f32()
//                 }
//                 _ => 0.0,
//             };
//
//             let pulse_freq = 1.0;
//             let pulse = color.scale_alpha(
//                 ((run_time  // t
//                         * pulse_freq * 2.0 * PI // f
//                     ).cos() // start at 1.0
//                     + 1.0 // shift range 0.0-2.0
//                 ) * 0.5 // scale range 0.0-1.0
//                     * 0.75  // scale range 0.0-0.75
//                     + 0.25, // shift range 0.25-1.0
//             );
//
//             container::transparent(t)
//                 .border(
//                     border::rounded(NODE_RADIUS)
//                         .color(pulse)
//                         .width(NODE_BORDER_WIDTH),
//                 )
//                 .background(iced::Color::from(app_theme.background.base_color))
//         };

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

// pub fn node_view<'a>(
//     node_instance: &'a ForayNodeInstance,
//     _id: u32,
//     _input_data: Dict<String, WireDataContainer<PortData>>,
// ) -> iced::Element<'a, WorkspaceMessage> {
//     let operation = |s| {
//         text(s)
//             .font(Font::with_name("DejaVu Math TeX Gyre"))
//             .size(30)
//             .into()
//     };
//     let trig = |s| {
//         text(s)
//             .size(20)
//             .font(Font::with_name("DejaVu Math TeX Gyre"))
//             .into()
//     };
//
//     match &node_instance.template {
//         ForayNodeTemplate::RustNode(rn) => match rn {
//             RustNodeTemplate::Constant(_value) => operation("not_implimented"), //constant::view(id, *value),
//             // RustNodeTemplate::Constant(_value) => operation("not_implimented"), //constant::view(id, *value),
//             // // RustNode::Linspace(linspace_config) => linspace_config.view(id),
//             // // RustNode::Plot(plot) => plot.view(id, input_data),
//             // // RustNode::Plot2D(plot) => plot.view(id, input_data),
//             // // RustNode::VectorField(vf) => vf.view(id, input_data),
//             // RustNodeTemplate::Add => operation("+"),
//             // RustNodeTemplate::Subtract => operation("−"),
//             // RustNodeTemplate::Multiply => operation("×"),
//             // RustNodeTemplate::Divide => operation("÷"),
//             // RustNodeTemplate::Cos => trig("cos(α)"),
//             // RustNodeTemplate::Sin => trig("sin(α)"),
//             // RustNodeTemplate::Sinc => trig("sinc(α)"),
//
//             _ => text(rn.to_string()).into(),
//         },
//         ForayNodeTemplate::PyNode(py_node) => text(py_node.name.clone()).into(),
//     }
// }
