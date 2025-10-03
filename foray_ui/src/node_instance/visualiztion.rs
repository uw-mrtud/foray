use std::f64::consts::{PI, TAU};

use derive_more::Display;
use foray_data_model::node::{ForayArray, PortData, PortType};
use foray_graph::graph::{Graph, GraphNode};
use iced::widget::image::Handle;

use ndarray::Array3;
use palette::{hsv, FromColor, Srgb};
use strum::VariantArray;

use crate::rust_nodes::RustNodeTemplate;

use super::ForayNodeInstance;
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Visualization {
    pub image_handle: Option<Handle>,
    pub parameters: VisualizationParameters,
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct VisualizationParameters {
    pub complex_map: RIMP,
}

#[derive(Clone, Copy, Debug, Display, PartialEq, Default, VariantArray)]
pub enum RIMP {
    Real,
    Imaginary,
    #[default]
    MagnitudeGray,
    MagnitudeLinear,
    Phase,
    PhaseRawHue,
    PhaseRawHueWeighted,
    PhaseWeighted,
}

impl Visualization {
    pub fn new(
        node_id: u32,
        graph: &Graph<ForayNodeInstance, PortType, PortData>,
        parameters: VisualizationParameters,
    ) -> Self {
        let mut visualization = Self {
            image_handle: None,
            parameters,
        };
        visualization.create_image_handle(node_id, graph);
        visualization
    }

    pub fn clear_hanlde(&mut self) {
        self.image_handle = None;
    }

    pub(crate) fn create_image_handle(
        &mut self,
        node_id: u32,
        graph: &Graph<ForayNodeInstance, PortType, PortData>,
    ) {
        let input_data = graph.get_input_data(&node_id);
        let output_data = graph.get_output_data(&node_id);
        let node = graph.get_node(node_id);

        let port_data = match node.template {
            super::ForayNodeTemplate::RustNode(RustNodeTemplate::Display) => {
                match input_data.get(node.inputs().iter().next().unwrap().0) {
                    Some(data) => Some(&*data.read().unwrap()),
                    None => None,
                }
            }
            _ => match node.outputs().iter().next() {
                Some((name, PortType::Array(_port_type, _shape))) => match output_data.get(name) {
                    Some(data) => Some(&*data.read().unwrap()),
                    None => None,
                },
                _ => None,
            },
        };

        self.image_handle = match port_data {
            Some(port) => {
                let data = match port {
                    PortData::Array(ForayArray::Float(a)) => Some(
                        Array3::<f64>::from_shape_vec(
                            (a.shape()[0], a.shape()[1], 3),
                            a.indexed_iter()
                                .flat_map(|(_, v)| [*v, *v, *v])
                                .collect::<Vec<_>>(),
                        )
                        .expect("square matrix"),
                    ),
                    PortData::Array(ForayArray::Complex(a)) => {
                        let max_mag = a.iter().map(|v| v.norm()).reduce(f64::max).unwrap_or(0.0);
                        Some(
                            Array3::<f64>::from_shape_vec(
                                (a.shape()[0], a.shape()[1], 3),
                                a.indexed_iter()
                                    .flat_map(|(_, v)| {
                                        // let normalized = v.norm().log10();
                                        match self.parameters.complex_map {
                                            RIMP::Real => [v.re; 3],
                                            RIMP::Imaginary => [v.im; 3],
                                            RIMP::MagnitudeGray => [v.norm() / max_mag; 3],
                                            RIMP::MagnitudeLinear => {
                                                linear_color_map(v.norm() / max_mag)
                                            }
                                            RIMP::Phase => {
                                                let angle = (v.im).atan2(v.re) + PI;
                                                // hsv_color_map(angle, v.norm())
                                                cyclic_color_map(angle)
                                            }
                                            RIMP::PhaseRawHue => {
                                                let angle = (v.im).atan2(v.re) + PI;
                                                hsv_color_map(angle, 1.0)
                                            }
                                            RIMP::PhaseRawHueWeighted => {
                                                let angle = (v.im).atan2(v.re) + PI;
                                                hsv_color_map(angle, v.norm() / max_mag)
                                            }
                                            RIMP::PhaseWeighted => {
                                                let angle = (v.im).atan2(v.re) + PI;
                                                weighted_cyclic_color_map(angle, v.norm() / max_mag)
                                            }
                                        }
                                    })
                                    .collect::<Vec<_>>(),
                            )
                            .expect("square matrix"),
                        )
                    }
                    _ => None,
                };
                data.map(|data| create_rgb_handle(&data))
            }
            _ => None,
        };
    }
}

//fn create_grayscale_handle(data: &Array3<f64>) -> Handle {}
fn create_rgb_handle(data: &Array3<f64>) -> Handle {
    // trace!("Creating image handle for plot2d, {:?}", data.shape());
    // let max = data.iter().fold(-f64::INFINITY, |a, &b| a.max(b));
    // let min = data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    // let min = 0.0;
    let brightness = |p: f64| {
        return (p * 255.0).round() as u8;
        // let p = ((p - min) / (max - min)) as f32;
        // let p = if p.is_nan() { 0.0 } else { p };
        // (p * 255.0).round() as u8
    };
    let img: Vec<u8> = data
        .outer_iter()
        .flat_map(|row| {
            row.outer_iter()
                .flat_map(|p| {
                    if p.len() == 1 {
                        let b = brightness(p[0]);
                        [b, b, b, 255]
                    } else if p.len() == 3 {
                        [brightness(p[0]), brightness(p[1]), brightness(p[2]), 255]
                    } else {
                        panic!("unsupported array dimensions")
                    }
                })
                .collect::<Vec<_>>()
        })
        .collect();
    Handle::from_rgba(data.dim().0 as u32, data.dim().1 as u32, img)
}

/// angle in radians
fn hsv_color_map(angle: f64, lightness: f64) -> [f64; 3] {
    let hsv: hsv::Hsv<_, f64> = hsv::Hsv::new(360.0 * angle / (TAU), 1.0, lightness);
    // let rgb: [f64; 3] = hsv.into();
    // rgb
    let (r, g, b) = Srgb::from_color(hsv).into_components();
    [r, g, b]
}

/// angle in positive radians, brightness 0.0-1.0
fn weighted_cyclic_color_map(angle: f64, brightness: f64) -> [f64; 3] {
    let img = include_bytes!("../../data/colormap/CET-C7.bin");
    let len = img.len() / 3;
    let cycles = (angle / TAU).fract();
    let r_index = (cycles * len as f64).floor() as usize * 3;

    let scale = (1.0 / 255.0) * brightness;
    [
        img[r_index] as f64 * scale,
        img[r_index + 1] as f64 * scale,
        img[r_index + 2] as f64 * scale,
    ]
}
fn cyclic_color_map(angle: f64) -> [f64; 3] {
    let img = include_bytes!("../../data/colormap/CET-C7.bin");
    let len = img.len() / 3;
    let cycles = (angle / TAU).fract();
    let r_index = (cycles * len as f64).floor() as usize * 3;

    let scale = 1.0 / 255.0;
    [
        img[r_index] as f64 * scale,
        img[r_index + 1] as f64 * scale,
        img[r_index + 2] as f64 * scale,
    ]
}
// value: 0.0 to 1.0
fn linear_color_map(value: f64) -> [f64; 3] {
    let img = include_bytes!("../../data/colormap/CET-L20.bin");
    let len = img.len() / 3;
    let r_index = (value * (len - 1) as f64).floor() as usize * 3;

    let scale = 1.0 / 255.0;
    [
        img[r_index] as f64 * scale,
        img[r_index + 1] as f64 * scale,
        img[r_index + 2] as f64 * scale,
    ]
}
