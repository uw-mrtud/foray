use foray_data_model::node::{ForayArray, PortData, PortType};
use foray_data_vis::series_vis::{SeriesVis, SeriesVisOptions};
use foray_graph::graph::{Graph, GraphNode};
use iced::widget::image::Handle;

use serde::{Deserialize, Serialize};

use crate::{
    node_instance::visualization_parameters::VisualizationParameters, rust_nodes::RustNodeTemplate,
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
            Visualization::Series(series_vis) => todo!(), //series_vis.svg = None,
        }
    }

    pub fn new_series(
        nx: u32,
        graph: &Graph<ForayNodeInstance, PortType, PortData>,
        parameters: SeriesVisOptions,
    ) -> Self {
        let input_data = graph.get_input_data(&nx);
        let node = graph.get_node(nx);

        let y_data: Vec<_> = node
            .inputs()
            .iter()
            .filter_map(|p| match input_data.get(p.0) {
                Some(data) => match &*data.read().unwrap() {
                    PortData::Array(foray_array) => match foray_array {
                        ForayArray::Integer(array_base) => todo!(),
                        ForayArray::Float(array_base) => {
                            if array_base.ndim() == 1 {
                                Some(
                                    array_base
                                        .clone()
                                        .into_shape_with_order(array_base.len())
                                        .unwrap(),
                                )
                            } else {
                                Default::default()
                            }
                        }
                        ForayArray::Complex(array_base) => todo!(),
                        _ => Default::default(),
                    },
                    _ => Default::default(),
                },
                None => Default::default(),
            })
            .collect();

        let y_max_length = y_data.iter().map(|y| y.len()).max().unwrap_or_default();

        let x_data = (0..y_max_length).map(|x| x as f64).collect();

        Visualization::Series(SeriesVis::new(x_data, y_data, parameters))
    }
}

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
