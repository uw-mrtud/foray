use std::f64::consts::{PI, TAU};

use foray_data_model::node::{ForayArray, PortData, PortType};
use foray_graph::graph::{Graph, GraphNode};
use iced::widget::image::Handle;

use palette::{hsv, FromColor, Srgb};
use serde::{Deserialize, Serialize};

use crate::{
    node_instance::{
        histogram::Histogram,
        visualization_parameters::{VisualizationParameters, RIMP},
    },
    rust_nodes::RustNodeTemplate,
};

use super::ForayNodeInstance;
#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct Visualization {
    #[serde(skip)]
    pub image_handle: Option<Handle>,
    pub parameters: VisualizationParameters,
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
        visualization.create_cached_data(node_id, graph);
        visualization
    }

    pub fn clear_hanlde(&mut self) {
        self.image_handle = None;
    }

    pub(crate) fn create_cached_data(
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

        match port_data {
            None => {
                self.image_handle = None;
                self.parameters.histogram = None;
            }
            Some(port_data) => {
                let dimensions = port_data.dimensions();
                self.parameters.update_dimension_lengths(dimensions);

                self.image_handle = port_data_to_image_handle(port_data, &self.parameters);
                self.parameters.histogram = Histogram::new(port_data)
            }
        }
    }
}

fn port_data_to_image_handle(
    port_data: &PortData,
    parameters: &VisualizationParameters,
) -> Option<Handle> {
    match port_data {
        PortData::Array(ForayArray::Float(a)) => {
            let array_slice = parameters.slice_array_2d(a);
            let (x_len, y_len) = parameters.xy_length();
            // let max_mag = array_slice
            //     .iter()
            //     .max_by(|a: &&f64, b: &&f64| a.total_cmp(b))
            //     .unwrap_or(&0.0);

            let img = array_slice
                .outer_iter()
                .flat_map(|row| {
                    row.iter()
                        .flat_map(|v| match parameters.complex_map {
                            RIMP::MagnitudeLinear => linear_color_map(parameters.map_value(*v)),
                            _ => linear_grayscale(parameters.map_value(*v)),
                        })
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>();
            Some(Handle::from_rgba(x_len as u32, y_len as u32, img))
        }
        PortData::Array(ForayArray::Complex(a)) => {
            let array_slice = parameters.slice_array_2d(a);
            let (x_len, y_len) = parameters.xy_length();
            // let max_mag = array_slice
            //     .iter()
            //     .map(|v| v.norm())
            //     .reduce(f64::max)
            //     .unwrap_or(0.0);

            let img = array_slice
                .outer_iter()
                .flat_map(|row| {
                    row.iter()
                        .flat_map(|v| match parameters.complex_map {
                            RIMP::Real => linear_grayscale(parameters.map_value(v.re)),
                            RIMP::Imaginary => linear_grayscale(parameters.map_value(v.im)),
                            RIMP::MagnitudeGray => linear_grayscale(parameters.map_value(v.norm())),
                            RIMP::MagnitudeLinear => {
                                linear_color_map(parameters.map_value(v.norm()))
                            }
                            RIMP::Phase => {
                                let angle = (v.im).atan2(v.re) + PI;
                                cyclic_color_map(angle)
                            }
                            RIMP::PhaseRawHue => {
                                let angle = (v.im).atan2(v.re) + PI;
                                hsv_color_map(angle, 1.0)
                            }
                            RIMP::PhaseRawHueWeighted => {
                                let angle = (v.im).atan2(v.re) + PI;
                                hsv_color_map(angle, parameters.map_value(v.norm()))
                            }
                            RIMP::PhaseWeighted => {
                                let angle = (v.im).atan2(v.re) + PI;
                                weighted_cyclic_color_map(angle, parameters.map_value(v.norm()))
                            }
                        })
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>();

            Some(Handle::from_rgba(x_len as u32, y_len as u32, img))
        }
        _ => None,
    }
}

/// angle in radians
fn hsv_color_map(angle: f64, lightness: f64) -> [u8; 4] {
    let hsv: hsv::Hsv<_, f64> = hsv::Hsv::new(360.0 * angle / (TAU), 1.0, lightness);
    let (r, g, b) = Srgb::from_color(hsv).into_format().into_components();
    [r, g, b, 255]
}

/// angle in positive radians, brightness 0.0-1.0
fn weighted_cyclic_color_map(angle: f64, brightness: f64) -> [u8; 4] {
    let img = include_bytes!("../../data/colormap/CET-C7.bin");
    let len = img.len() / 3;
    let cycles = (angle / TAU).fract();
    let r_index = (cycles * len as f64).floor() as usize * 3;

    [
        (img[r_index] as f64 * brightness) as u8,
        (img[r_index + 1] as f64 * brightness) as u8,
        (img[r_index + 2] as f64 * brightness) as u8,
        255,
    ]
}

fn cyclic_color_map(angle: f64) -> [u8; 4] {
    let img = include_bytes!("../../data/colormap/CET-C7.bin");
    let len = img.len() / 3;
    let cycles = (angle / TAU).fract();
    let r_index = (cycles * len as f64).floor() as usize * 3;

    [img[r_index], img[r_index + 1], img[r_index + 2], 255]
}

// value: 0.0 to 1.0
fn linear_color_map(value: f64) -> [u8; 4] {
    let img = include_bytes!("../../data/colormap/CET-L20.bin");
    let len = img.len() / 3;
    let r_index = (value * (len - 1) as f64).floor() as usize * 3;

    [img[r_index], img[r_index + 1], img[r_index + 2], 255]
}

fn linear_grayscale(value: f64) -> [u8; 4] {
    let gray_level = (value * 255.0).round() as u8;
    [gray_level, gray_level, gray_level, 255]
}
