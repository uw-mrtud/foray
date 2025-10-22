use std::f64::consts::PI;

use derive_more::Display;
use foray_data_model::node::{ForayArray, PortData};
use iced::{Alignment::Center, Element, Length};

use numpy::Complex64;
use serde::{Deserialize, Serialize};
use strum::{VariantArray, VariantNames};

use crate::{
    node_instance::{histogram::Histogram, visualization_parameters::VisualizationParameters},
    workspace::WorkspaceMessage,
};
use iced::widget::{column, *};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ValueMapping {
    pub floor: f32,
    pub ceil: f32,
    pub color_map: ColorMap,
    #[serde(skip)]
    pub histogram: Option<Histogram>,
}

#[derive(Clone, Copy, Debug, Display, PartialEq, Serialize, Deserialize)]
pub enum ColorMap {
    Complex(RIMP),
    Real(RealMap),
}

#[derive(Clone, Copy, Debug, PartialEq, VariantNames, Serialize, Deserialize)]
pub enum RIMP {
    Real(RealMap),
    Imag(RealMap),
    Mag(RealMap),
    Phase(CyclicMap),
}

impl std::fmt::Display for RIMP {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RIMP::Real(_) => write!(f, "Real"),
            RIMP::Imag(_) => write!(f, "Imag"),
            RIMP::Mag(_) => write!(f, "Mag"),
            RIMP::Phase(_) => write!(f, "Phase"),
        }
    }
}

#[derive(Clone, Copy, Debug, Display, PartialEq, Default, VariantArray, Serialize, Deserialize)]
pub enum RealMap {
    Gray,
    #[default]
    Color,
}

impl RealMap {
    fn map_color(&self, v: f64) -> [u8; 4] {
        match self {
            RealMap::Gray => linear_grayscale(v),
            RealMap::Color => linear_color_map(v),
        }
    }
}

#[derive(Clone, Copy, Debug, Display, PartialEq, Default, VariantArray, Serialize, Deserialize)]
pub enum CyclicMap {
    Cyclic,
    #[default]
    Weighted,
}
impl CyclicMap {
    fn map_color(&self, cycles: f64, brightness: f64) -> [u8; 4] {
        match self {
            CyclicMap::Cyclic => cyclic_color_map(cycles),
            CyclicMap::Weighted => weighted_cyclic_color_map(cycles, brightness),
        }
    }
}

impl Default for ValueMapping {
    fn default() -> Self {
        Self {
            floor: 0.0,
            ceil: 1.0,
            color_map: ColorMap::Real(RealMap::Color),
            histogram: None,
        }
    }
}

impl ValueMapping {
    pub(super) fn create_histogram(&mut self, port_data: &PortData) {
        self.histogram = Histogram::new(port_data, &self);
    }

    // clamp floor and ceil based on histogram max/min
    pub(super) fn clamp_floor_ceil(&mut self) {
        match self.color_map {
            ColorMap::Complex(RIMP::Phase(CyclicMap::Cyclic)) => {
                self.floor = 0.0;
                self.ceil = 1.0;
            }
            _ => {
                self.floor = self
                    .floor
                    .max(self.histogram.as_ref().map(|h| h.min).unwrap_or(0.0));
                self.ceil = self.ceil.min(
                    self.histogram
                        .as_ref()
                        .map(|h| h.max)
                        .unwrap_or(1.0)
                        // force ceil to be at least slightly larger than floor
                        .max(self.floor + 0.00001),
                );
            }
        }
    }

    /// map a raw data value to between 0.0 and 1.0 based on current floor/ciel setting
    pub(crate) fn map_value(&self, x: f64) -> f64 {
        let (floor, ceil) = (self.floor as f64, self.ceil as f64);

        let m = 1.0 / (ceil - floor);
        let b = floor / (floor - ceil);
        let y = m * x + b;
        y.clamp(0.0, 1.0)
    }

    /// map a value between 0.0 and 1.0 to a color value
    pub fn color_map_real(&self, v: f64) -> [u8; 4] {
        match self.color_map {
            ColorMap::Real(real_map) => match real_map {
                RealMap::Gray => linear_grayscale(self.map_value(v)),
                RealMap::Color => linear_color_map(self.map_value(v)),
            },
            // When drawing histogram, we map a real value, even if the underlying data is complex
            ColorMap::Complex(rimp) => match rimp {
                RIMP::Real(rmap) => rmap.map_color(self.map_value(v)),
                RIMP::Imag(rmap) => rmap.map_color(self.map_value(v)),
                RIMP::Mag(rmap) => rmap.map_color(self.map_value(v)),
                RIMP::Phase(cymap) => match cymap {
                    // Cylcic color, show the difference in hue
                    CyclicMap::Cyclic => cymap.map_color(v, 1.0),
                    // Weighted colors, show the difference in brighntes
                    CyclicMap::Weighted => RealMap::Gray.map_color(v),
                },
            },
        }
    }

    fn complex_to_cycles(v: Complex64) -> f64 {
        ((v.im).atan2(v.re) + PI) / (2.0 * PI)
    }

    /// map a raw complex value a color value
    pub fn color_map_complex(&self, v: Complex64) -> [u8; 4] {
        match self.color_map {
            ColorMap::Complex(rimp) => match rimp {
                RIMP::Real(rmap) => rmap.map_color(self.map_value(v.re)),
                RIMP::Imag(rmap) => rmap.map_color(self.map_value(v.im)),
                RIMP::Mag(rmap) => rmap.map_color(self.map_value(v.norm())),
                RIMP::Phase(cymap) => match cymap {
                    CyclicMap::Cyclic => {
                        cymap.map_color(self.map_value(Self::complex_to_cycles(v)), 1.0)
                    }
                    CyclicMap::Weighted => {
                        cymap.map_color(Self::complex_to_cycles(v), self.map_value(v.norm()))
                    }
                },
            },
            ColorMap::Real(_real_map) => todo!(),
        }
    }

    /// map a raw complex value a scaler value
    pub fn value_map_complex(&self, v: Complex64) -> f64 {
        match self.color_map {
            ColorMap::Complex(rimp) => match rimp {
                RIMP::Real(_rmap) => v.re,
                RIMP::Imag(_rmap) => v.im,
                RIMP::Mag(_rmap) => v.norm(),
                RIMP::Phase(cymap) => match cymap {
                    CyclicMap::Cyclic => Self::complex_to_cycles(v),
                    CyclicMap::Weighted => v.norm(),
                },
            },
            ColorMap::Real(_real_map) => todo!(),
        }
    }

    pub fn view<'a>(
        &'a self,
        visualization_parameters: &'a VisualizationParameters,
        node_id: u32,
        pick_list_size: f32,
    ) -> Element<'a, WorkspaceMessage> {
        let rimp_widget: Element<WorkspaceMessage> = match self.color_map {
            ColorMap::Complex(rimp) => {
                let (default_rmap, default_cymap) = match rimp {
                    RIMP::Real(real_map) => (real_map, Default::default()),
                    RIMP::Imag(real_map) => (real_map, Default::default()),
                    RIMP::Mag(real_map) => (real_map, Default::default()),
                    RIMP::Phase(cyclic_map) => (Default::default(), cyclic_map),
                };

                let rimp_variants = [
                    RIMP::Real(default_rmap),
                    RIMP::Imag(default_rmap),
                    RIMP::Mag(default_rmap),
                    RIMP::Phase(default_cymap),
                ];

                let value_map_pick = pick_list(rimp_variants, Some(rimp), move |value| {
                    WorkspaceMessage::UpdateVisualization(
                        node_id,
                        VisualizationParameters {
                            value_mapping: ValueMapping {
                                color_map: ColorMap::Complex(value),
                                histogram: None,
                                ..self.clone()
                            },
                            ..visualization_parameters.clone()
                        },
                    )
                })
                .text_size(pick_list_size);
                let color_map_pick: Element<WorkspaceMessage> = match rimp {
                    RIMP::Real(real_map) | RIMP::Imag(real_map) | RIMP::Mag(real_map) => {
                        pick_list(RealMap::VARIANTS, Some(real_map), move |value| {
                            WorkspaceMessage::UpdateVisualization(
                                node_id,
                                VisualizationParameters {
                                    value_mapping: ValueMapping {
                                        color_map: ColorMap::Real(value),
                                        ..self.clone()
                                    },
                                    ..visualization_parameters.clone()
                                },
                            )
                        })
                        .text_size(pick_list_size)
                        .into()
                    }

                    RIMP::Phase(cyclic_map) => {
                        pick_list(CyclicMap::VARIANTS, Some(cyclic_map), move |value| {
                            let (new_floor, new_ceil) = match value {
                                CyclicMap::Cyclic => (0.0, 1.0),
                                CyclicMap::Weighted => (self.floor, self.ceil),
                            };
                            WorkspaceMessage::UpdateVisualization(
                                node_id,
                                VisualizationParameters {
                                    value_mapping: ValueMapping {
                                        color_map: ColorMap::Complex(RIMP::Phase(value)),
                                        histogram: None,
                                        floor: new_floor,
                                        ceil: new_ceil,
                                    },
                                    ..visualization_parameters.clone()
                                },
                            )
                        })
                        .text_size(pick_list_size)
                        .into()
                    }
                };

                row![value_map_pick, color_map_pick].spacing(4.0).into()
            }
            ColorMap::Real(real_map) => {
                pick_list(RealMap::VARIANTS, Some(real_map), move |value| {
                    WorkspaceMessage::UpdateVisualization(
                        node_id,
                        VisualizationParameters {
                            value_mapping: ValueMapping {
                                color_map: ColorMap::Real(value),
                                ..self.clone()
                            },
                            ..visualization_parameters.clone()
                        },
                    )
                })
                .text_size(pick_list_size)
                .into()
            }
        };

        let value_mapping_widgets: Element<'_, WorkspaceMessage> = match &visualization_parameters
            .value_mapping
            .histogram
        {
            Some(histogram) => {
                let histogram_view = canvas(self);

                let n_steps = 50;
                let step_size = (histogram.max - histogram.min) / n_steps as f32;
                let (floor_row, ceil_row): (Element<WorkspaceMessage>, Element<WorkspaceMessage>) =
                    // Don't create floor/ceil controls for clyclic maps
                    match self.color_map {
                        ColorMap::Complex(RIMP::Phase(CyclicMap::Cyclic)) => {
                            (horizontal_space().into(), horizontal_space().into())
                        }
                        _ => (// Floor Row
                            row![
                                text("floor").width(40),
                                slider(histogram.min..=histogram.max, self.floor, move |new_floor| {
                                    let mut new_parameters = visualization_parameters.clone();
                                    new_parameters.value_mapping.floor = f32::min(new_floor,histogram.max - step_size);
                                    new_parameters.value_mapping.ceil = f32::max(visualization_parameters.value_mapping.ceil,new_floor+step_size);
                                    WorkspaceMessage::UpdateVisualization(node_id, new_parameters)
                                })
                                .step(step_size)
                            ]
                            .align_y(Center)
                            .into(),
                            // Ceil Row 
                            row![
                                text("ceil").width(40),
                                slider(histogram.min..=histogram.max, self.ceil, move |new_ceil| {
                                    let mut new_parameters = visualization_parameters.clone();
                                    new_parameters.value_mapping.ceil = f32::max(new_ceil,histogram.min + step_size);
                                    new_parameters.value_mapping.floor = f32::min(visualization_parameters.value_mapping.floor,new_ceil-step_size);
                                    WorkspaceMessage::UpdateVisualization(node_id, new_parameters)
                                })
                                .step(step_size)
                            ]
                            .align_y(Center)
                            .into(),
                        ),
                    };

                column![
                    row!["Value Mapping"],
                    horizontal_rule(1.0),
                    container(histogram_view.width(Length::Fill).height(50.0)).padding(6.0),
                    floor_row,
                    ceil_row,
                    row![rimp_widget].align_y(Center),
                ]
                .into()
            }
            None => column![].into(),
        };
        value_mapping_widgets
    }

    /// Sets color map to a valid type given a new port_Data type.
    /// Trys to match the old color map as closely as possible
    pub(crate) fn enforce_color_map_constaints(
        &mut self,
        port_data: &foray_data_model::node::PortData,
    ) {
        match port_data {
            PortData::Array(foray_array) => match foray_array {
                ForayArray::Integer(_) | ForayArray::Float(_) | ForayArray::Boolean(_) => {
                    match self.color_map {
                        crate::node_instance::value_mapping::ColorMap::Real(_rimp) => {}
                        crate::node_instance::value_mapping::ColorMap::Complex(rimp) => {
                            match rimp {
                                RIMP::Real(real_map)
                                | RIMP::Imag(real_map)
                                | RIMP::Mag(real_map) => {
                                    self.color_map = ColorMap::Real(real_map);
                                }
                                RIMP::Phase(_cyclic_map) => {
                                    self.color_map = ColorMap::Real(RealMap::default());
                                }
                            }
                        }
                    }
                }

                ForayArray::Complex(_) => match self.color_map {
                    crate::node_instance::value_mapping::ColorMap::Complex(_rimp) => {}
                    crate::node_instance::value_mapping::ColorMap::Real(real_map) => {
                        self.color_map = ColorMap::Complex(RIMP::Mag(real_map))
                    }
                },

                _ => {}
            },
            _ => {}
        };
    }
}

/// angle in radians
// fn hsv_color_map(angle: f64, lightness: f64) -> [u8; 4] {
//     let hsv: hsv::Hsv<_, f64> = hsv::Hsv::new(360.0 * angle / (TAU), 1.0, lightness);
//     let (r, g, b) = Srgb::from_color(hsv).into_format().into_components();
//     [r, g, b, 255]
// }

/// angle in positive radians, brightness 0.0-1.0
fn weighted_cyclic_color_map(cycles: f64, brightness: f64) -> [u8; 4] {
    let img = include_bytes!("../../data/colormap/CET-C7.bin");
    let len = img.len() / 3;
    let parital_cycles = (cycles).fract();
    let r_index = (parital_cycles * len as f64).floor() as usize * 3;

    [
        (img[r_index] as f64 * brightness) as u8,
        (img[r_index + 1] as f64 * brightness) as u8,
        (img[r_index + 2] as f64 * brightness) as u8,
        255,
    ]
}

fn cyclic_color_map(cycles: f64) -> [u8; 4] {
    let img = include_bytes!("../../data/colormap/CET-C7.bin");
    let len = img.len() / 3;
    let partial_cycles = (cycles).fract();
    let r_index = (partial_cycles * len as f64).floor() as usize * 3;

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
