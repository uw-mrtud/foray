use std::f32::consts::PI;

use super::{PortData, RustNode};
use crate::app::Message;
use crate::gui_node::PortDataContainer;
use crate::interface::node::{INNER_NODE_WIDTH, NODE_BORDER_WIDTH};
use crate::math::{linspace_delta, Vector};
use crate::nodes::UINodeTemplate;
use crate::StableMap;
use colorgrad::Gradient;
use glam::{Mat3, Vec3};
use iced::mouse;
use iced::widget::canvas::path::Builder;
use iced::widget::canvas::{Path, Stroke};
use iced::widget::{container, horizontal_space, row, text, text_input};
use iced::Alignment::Center;
use iced::{
    widget::{canvas, column},
    Element,
};
use iced::{Rectangle, Renderer, Theme};
use itertools::Itertools;
use ndarray::Array3;
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
            width: 64.,
            height: 64., // * (INNER_NODE_HEIGHT / INNER_NODE_WIDTH),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct VectorField {
    rect: Rect,
    z_index: usize,
}

impl VectorField {
    pub fn view<'a>(
        &self,
        _id: u32,
        input_data: StableMap<String, PortDataContainer>,
    ) -> Element<'a, Message> {
        match input_data.get("a") {
            Some(port) => {
                let data = match (**port).read().unwrap().clone() {
                    PortData::ArrayComplex(a) => Array3::<f64>::from_shape_vec(
                        [
                            (a.len() as f32).sqrt() as usize,
                            (a.len() as f32).sqrt() as usize,
                            3,
                        ],
                        a.iter()
                            .flat_map(|v| {
                                let (r, theta) = v.to_polar();
                                [theta.sin(), theta.cos(), r]
                            })
                            .collect::<Vec<_>>(),
                    )
                    .expect("square matrix"),
                    PortData::ArrayReal(a) => {
                        let xy_len = (a.len() as f32 / 3.0).sqrt() as usize;
                        a.into_shape_with_order([xy_len, xy_len, 3])
                            .unwrap_or_else(|e| panic!("Unexpected shape{e}"))
                    }
                    _ => panic!("unsuported plot types {:?}", port),
                };
                container(
                    canvas(VectorFieldCanvas {
                        data,
                        config: *self,
                    })
                    .width(INNER_NODE_WIDTH * 2.)
                    .height(INNER_NODE_WIDTH * 2.),
                )
                .padding(NODE_BORDER_WIDTH)
                .into()
            }
            None => container(
                container(text("n/a"))
                    .width(INNER_NODE_WIDTH * 2.)
                    .height(INNER_NODE_WIDTH * 2.),
            )
            .padding(NODE_BORDER_WIDTH)
            .into(),
        }
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
            Message::UpdateNodeTemplate(
                id,
                //TODO: handle z_index
                UINodeTemplate::RustNode(RustNode::VectorField(VectorField { rect, z_index: 0 })),
            )
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
struct VectorFieldCanvas {
    data: Array3<f64>,
    config: VectorField,
}

impl<Message> canvas::Program<Message> for VectorFieldCanvas {
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
        // Scale for the conifgured height/width
        frame.scale_nonuniform(scale);

        // Move the center point to the center of our canvas
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

        let vec_scale = 1.5;

        self.data
            .indexed_iter()
            // This looks nicer(don't need to do the chunk.collect() weirdness below) with
            // the nightly iter_array_chunks feature
            .chunks(3)
            .into_iter()
            .map(|chunk| {
                let [((x, y, _), vx), ((_, _, _), vy), ((_, _, _), vz)] =
                    chunk.collect::<Vec<_>>()[..]
                else {
                    panic!("array3 is in an unexpected format")
                };

                let v = Vec3::from([
                    *vx as f32 * vec_scale,
                    *vy as f32 * vec_scale,
                    *vz as f32 * vec_scale,
                ]);

                let arrow_angle = PI / 8.0;
                let arrow_left = Mat3::from_rotation_z(arrow_angle) * (v * 0.8);
                let arrow_right = Mat3::from_rotation_z(-arrow_angle) * (v * 0.8);

                let v_tail = Vec3::from([x as f32 - 5.0, y as f32 - 5.0, 0.0]);
                let v_tip = v_tail + v;
                let tip_left = v_tail + arrow_left;
                let tip_right = v_tail + arrow_right;

                let mut path = Builder::new();
                path.move_to((v_tail[0], v_tail[1]).into());
                path.line_to((v_tip[0], v_tip[1]).into());
                path.line_to((tip_left[0], tip_left[1]).into());
                path.move_to((v_tip[0], v_tip[1]).into());
                path.line_to((tip_right[0], tip_right[1]).into());
                let color = colorgrad::preset::spectral().at((*vz as f32 + 1.0) / 2.0);
                (
                    path.build(),
                    //TODO: change line stroke based on z
                    Stroke::default()
                        .with_color(iced::Color {
                            r: color.r,
                            g: color.g,
                            b: color.b,
                            a: 1.0,
                        })
                        .with_line_join(canvas::LineJoin::Miter)
                        .with_width(1.),
                )
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
