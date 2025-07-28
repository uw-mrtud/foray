use super::{PortData, RustNode};
use crate::app::Message;
use crate::gui_node::PortDataContainer;
use crate::interface::node::{INNER_NODE_HEIGHT, INNER_NODE_WIDTH, NODE_BORDER_WIDTH};
use crate::math::{linspace_delta, Vector};
use crate::nodes::UINodeTemplate;
use crate::StableMap;
use iced::widget::canvas::{Path, Stroke};
use iced::widget::{container, horizontal_space, row, text, text_input};
use iced::Alignment::Center;
use iced::{mouse, Point};
use iced::{
    widget::{canvas, column},
    Element,
};
use iced::{Rectangle, Renderer, Theme};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

// Rectanlge specified by center position, width and height
// y is up
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct Rect {
    pub center: Vector,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn right(&self) -> f32 {
        self.center.x + self.width / 2.
    }
    pub fn left(&self) -> f32 {
        self.center.x - self.width / 2.
    }
    pub fn top(&self) -> f32 {
        self.center.y + self.height / 2.
    }
    pub fn bottom(&self) -> f32 {
        self.center.y - self.height / 2.
    }
}
impl Default for Rect {
    fn default() -> Self {
        Rect {
            center: [0., 0.].into(),
            width: 20.,
            height: 20. * (INNER_NODE_HEIGHT / INNER_NODE_WIDTH),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct Plot {
    rect: Rect,
}

impl Plot {
    pub fn view<'a>(
        &self,
        _id: u32,
        input_data: StableMap<String, PortDataContainer>,
    ) -> Element<'a, Message> {
        let (x, y) =
            if let (Some(x_port), Some(y_port)) = (input_data.get("x"), input_data.get("y")) {
                if let (PortData::ArrayReal(x), PortData::ArrayReal(y)) = (
                    x_port.read().unwrap().clone(),
                    y_port.read().unwrap().clone(),
                ) {
                    (
                        x.into_raw_vec_and_offset()
                            .0
                            .into_iter()
                            .map(|f| f as f32)
                            .collect(),
                        y.into_raw_vec_and_offset()
                            .0
                            .into_iter()
                            .map(|f| f as f32)
                            .collect(),
                    )
                } else {
                    panic!("unsuported plot types ")
                }
            } else {
                (vec![], vec![])
            };
        container(
            canvas(PlotCanvas {
                x,
                y,
                config: *self,
            })
            .width(INNER_NODE_WIDTH * 2.)
            .height(INNER_NODE_HEIGHT * 2.),
        )
        .padding(NODE_BORDER_WIDTH)
        .into()
    }

    pub fn config_view(
        &self,
        id: u32,
        _input_data: StableMap<String, PortDataContainer>,
    ) -> Option<Element<'_, Message>> {
        let center = self.rect.center;
        let width = self.rect.width;
        let height = self.rect.height;
        let message = move |rect| {
            Message::UpdateNodeTemplate(id, UINodeTemplate::RustNode(RustNode::Plot(Plot { rect })))
        };
        Some(
            column![
                row![
                    text("center:"),
                    horizontal_space(),
                    text("x"),
                    text_input("0", &center.x.to_string()).on_input(move |value| {
                        let mut n = self.rect;
                        n.center.x = value.parse().unwrap_or(0.);
                        message(n)
                    }),
                    text("y"),
                    text_input("0", &center.y.to_string()).on_input(move |value| {
                        let mut n = self.rect;
                        n.center.y = value.parse().unwrap_or(0.);
                        message(n)
                    }),
                ]
                .align_y(Center)
                .spacing(4.),
                row![
                    text("width:"),
                    horizontal_space(),
                    text_input("0", &width.to_string()).on_input(move |value| {
                        let mut n = self.rect;
                        n.width = value.parse().unwrap_or(1.0f32).max(0.001);
                        message(n)
                    }),
                ]
                .align_y(Center),
                row![
                    text("height:"),
                    horizontal_space(),
                    text_input("0", &height.to_string()).on_input(move |value| {
                        let mut n = self.rect;
                        n.height = value.parse().unwrap_or(1.0f32).max(0.001);
                        message(n)
                    }),
                ]
                .align_y(Center)
            ]
            .spacing(5.0)
            .into(),
        )
    }
}

#[derive(Debug)]
struct PlotCanvas {
    x: Vec<f32>,
    y: Vec<f32>,
    config: Plot,
}

impl<Message> canvas::Program<Message> for PlotCanvas {
    // No internal state
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        // We prepare a new `Frame`
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        let node_width = bounds.width;
        let node_height = bounds.height;
        let aspect = node_width / node_height;

        frame.push_transform();
        //TODO: figure out how ot rephrase this in math, so I can actually
        //predict/undestand how to switch between plot space and screen space
        //// center canvas on the origin, y up
        frame.translate([frame.center().x, frame.center().y].into());
        frame.scale_nonuniform([1., -1.]);

        frame.push_transform();

        let scale = Vector::new(
            node_width / self.config.rect.width,
            node_height / self.config.rect.height,
        );
        // scale for the conifgured height/width
        frame.scale_nonuniform(scale);

        //move the center point to the center of our canvas
        frame.translate((-self.config.rect.center).into());

        // The frame is now centered on center, and goes from:
        // rect.left   -> rect.right
        // rect.bottom -> rect.top

        //// Grid
        {
            let color = theme.extended_palette().secondary.strong.color;
            let main_grid_stroke = Stroke::default()
                .with_width(0.8)
                .with_color(color.scale_alpha(0.5));

            let secondary_grid_stroke = main_grid_stroke.with_width(0.4);

            let tertiary_grid_strok = secondary_grid_stroke;

            let max_length = (self.config.rect.width + self.config.rect.center.x.abs())
                .max(self.config.rect.height + self.config.rect.center.y.abs());

            grid_path(self.config.rect, max_length, aspect, max_length)
                .into_iter()
                .for_each(|p| frame.stroke(&p, main_grid_stroke));

            {
                grid_path(self.config.rect, 0.05, aspect, 10.)
                    .into_iter()
                    .for_each(|p| frame.stroke(&p, secondary_grid_stroke));

                grid_path(self.config.rect, 0.025, aspect, 1.)
                    .into_iter()
                    .for_each(|p| frame.stroke(&p, tertiary_grid_strok));
            }
        }

        let line_stroke = Stroke::default()
            .with_color(theme.extended_palette().success.strong.color)
            .with_width(2.);
        self.x
            .clone()
            .into_iter()
            .zip(self.y.clone())
            .tuple_windows()
            .map(|(from, to)| {
                if from.0.is_finite() && from.1.is_finite() && to.0.is_finite() && to.1.is_finite()
                {
                    (Path::line(Point::from(from), Point::from(to)), line_stroke)
                } else if from.0.is_finite() && to.0.is_finite() {
                    (
                        Path::line(
                            Point::from((from.0, self.config.rect.center.y)),
                            Point::from((to.0, self.config.rect.center.y)),
                        ),
                        line_stroke.with_color(theme.palette().danger),
                    )
                } else {
                    (
                        Path::circle(
                            Point::from((
                                self.config.rect.right() - 1.,
                                self.config.rect.top() - 1.,
                            )),
                            0.75,
                        ),
                        line_stroke.with_color(theme.palette().danger),
                    )
                }
            })
            .for_each(|(path, stroke)| frame.stroke(&path, stroke));

        frame.pop_transform();

        vec![frame.into_geometry()]
    }
}

fn grid_path(
    plot_rect: Rect,
    tick_length_node_space: f32,
    aspect: f32,
    tick_spacing: f32,
) -> Vec<Path> {
    let left = ((plot_rect.left() / tick_spacing).floor()) * tick_spacing;
    let right = ((plot_rect.right() / tick_spacing).ceil()) * tick_spacing;
    let bottom = ((plot_rect.bottom() / tick_spacing).floor()) * tick_spacing;
    let top = ((plot_rect.top() / tick_spacing).ceil()) * tick_spacing;
    let cx = 0.0f32.clamp(left, right);
    let cy = 0.0f32.clamp(bottom, top);
    let x_tick_length = tick_length_node_space * plot_rect.height * aspect;
    let y_tick_length = tick_length_node_space * plot_rect.width;

    if left.is_nan() || right.is_nan() || top.is_nan() || bottom.is_nan() {
        panic!("Encountered nan!{:?}", (plot_rect, tick_spacing))
    }

    let h_lines = linspace_delta(top, bottom, tick_spacing)
        .into_iter()
        .map(|y| {
            if y.is_nan() {
                panic!("Encountered nan!{:?}", (plot_rect, tick_spacing))
            }
            Path::line(
                (cx - (y_tick_length / 2.), y).into(),
                (cx + (y_tick_length / 2.), y).into(),
            )
        });

    let v_lines = linspace_delta(right, left, tick_spacing)
        .into_iter()
        .map(|x| {
            if x.is_nan() {
                panic!("Encountered nan!{:?}", (plot_rect, tick_spacing))
            }

            Path::line(
                (x, cy - (x_tick_length / 2.)).into(),
                (x, cy + (x_tick_length / 2.)).into(),
            )
        });

    h_lines.chain(v_lines).collect()
}
