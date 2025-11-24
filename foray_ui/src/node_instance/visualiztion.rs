use foray_data_model::node::{ForayArray, PortData, PortType};
use foray_graph::graph::{Graph, GraphNode};
use iced::widget::image::Handle;

use plotters::{
    chart::ChartBuilder,
    prelude::IntoDrawingArea,
    series::LineSeries,
    style::{RGBColor, ShapeStyle, WHITE},
};
use serde::{Deserialize, Serialize};

use crate::{
    node_instance::visualization_parameters::VisualizationParameters, rust_nodes::RustNodeTemplate,
    style::theme::AppTheme,
};

use super::ForayNodeInstance;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Visualization {
    NDimVis(NDimVis),
    Series(SeriesVis),
}

impl Visualization {
    pub(crate) fn clear(&mut self) {
        match self {
            Visualization::NDimVis(ndim_vis) => ndim_vis.image_handle = None,
            Visualization::Series(series_vis) => series_vis.svg = None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct SeriesVis {
    #[serde(skip)]
    pub svg: Option<iced::advanced::svg::Svg>,
    pub parameters: SeriesParameters,
}
impl SeriesVis {
    pub(crate) fn new(
        nx: u32,
        graph: &Graph<ForayNodeInstance, PortType, PortData>,
        parameters: SeriesParameters,
    ) -> Self {
        let mut vis = Self {
            svg: None,
            parameters,
        };
        vis.create_cached_data(nx, graph);
        vis
    }

    fn create_cached_data(
        &mut self,
        node_id: u32,
        graph: &Graph<ForayNodeInstance, PortType, PortData>,
    ) {
        let input_data = graph.get_input_data(&node_id);
        let node = graph.get_node(node_id);

        let port_data = match input_data.get(node.inputs().iter().next().unwrap().0) {
            Some(data) => Some(&*data.read().unwrap()),
            None => None,
        };
        // dbg!(port_data);

        // Enforce constraints on visualization given a new port_data, wich may have a different
        // type
        //
        // This is a bit messy, but I don't currently have a better approach in mind.
        match port_data {
            None => {
                self.svg = None;
            }
            Some(port_data) => {
                self.svg = Self::port_data_to_svg(port_data);
            }
        }
    }

    fn port_data_to_svg(port_data: &PortData) -> Option<iced::advanced::svg::Svg> {
        let mut svg_buffer = String::new();
        match port_data {
            PortData::Array(ForayArray::Float(a)) => {
                let root_drawing_area =
                    plotters::backend::SVGBackend::with_string(&mut svg_buffer, (3000, 3000))
                        .into_drawing_area();

                let default_theme = AppTheme::default();

                let fg_color = default_theme.text.base_color.into_rbg8();
                let fg_color = RGBColor(fg_color.0, fg_color.1, fg_color.2);

                let series_color = default_theme.blue.weak_color().into_rbg8();
                let series_color = RGBColor(series_color.0, series_color.1, series_color.2);

                let mut chart = ChartBuilder::on(&root_drawing_area)
                    .margin_top(150)
                    .margin_right(150)
                    .x_label_area_size(300)
                    .y_label_area_size(300)
                    .build_cartesian_2d(0.0..(a.len() as f64), -1.2..1.2)
                    .unwrap();

                chart
                    .configure_mesh()
                    .disable_x_mesh()
                    .disable_y_mesh()
                    .label_style(("sans-serif", 140, &fg_color))
                    .x_label_formatter(&|x| format!("{:.0}", x))
                    .set_all_tick_mark_size(20)
                    .axis_style(ShapeStyle {
                        color: fg_color.into(),
                        filled: true,
                        stroke_width: 10,
                    })
                    .draw()
                    .unwrap();

                chart
                    .draw_series(LineSeries::new(
                        a.iter().enumerate().map(|(i, x)| (i as f64, *x)),
                        ShapeStyle {
                            color: series_color.into(),
                            filled: false,
                            stroke_width: 20,
                        },
                    ))
                    .unwrap();

                chart
                    .configure_series_labels()
                    .label_font(("sans-serif", 20, &WHITE))
                    .draw()
                    .unwrap();
            }
            _ => {
                return None;
            }
        };
        let dynamic_svg = iced::advanced::svg::Svg::new(iced::advanced::svg::Handle::from_memory(
            svg_buffer.into_bytes(),
        ));
        Some(dynamic_svg)
    }
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct SeriesParameters {}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct NDimVis {
    #[serde(skip)]
    pub image_handle: Option<Handle>,
    pub parameters: VisualizationParameters,
}

impl NDimVis {
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

        // Enforce constraints on visualization given a new port_data, wich may have a different
        // type
        //
        // This is a bit messy, but I don't currently have a better approach in mind.
        match port_data {
            None => {
                self.image_handle = None;
                self.parameters.value_mapping.histogram = None;
            }
            Some(port_data) => {
                let dimensions = port_data.dimensions();
                self.parameters.update_dimension_lengths(dimensions);

                self.parameters
                    .value_mapping
                    .enforce_color_map_constaints(&port_data);
                self.parameters.value_mapping.create_histogram(port_data);
                self.parameters.value_mapping.clamp_floor_ceil();

                self.image_handle = port_data_to_image_handle(port_data, &self.parameters);
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
                        .flat_map(
                            |v| parameters.value_mapping.color_map_real(*v), //     match parameters.complex_map {
                                                                             //     RIMP::MagColor => linear_color_map(parameters.map_value(*v)),
                                                                             //     _ => linear_grayscale(parameters.map_value(*v)),
                                                                             // }
                        )
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
                        .flat_map(|v| parameters.value_mapping.color_map_complex(*v))
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>();

            Some(Handle::from_rgba(x_len as u32, y_len as u32, img))
        }
        _ => None,
    }
}
