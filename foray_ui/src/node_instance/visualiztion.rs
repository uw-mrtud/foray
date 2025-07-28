use std::collections::BTreeMap;

use foray_data_model::node::{ForayArray, PortData, PortType};
use foray_graph::graph::GraphNode;
use iced::widget::image::Handle;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Visualization {
    pub image_handle: Option<Handle>,
}

impl Visualization {
    pub fn new(node: &ForayNodeInstance, output_data: &BTreeMap<String, PortData>) -> Self {
        let image_handle = match node.outputs().iter().next() {
            Some((name, PortType::Array(port_type, shape))) => match **port_type {
                PortType::Float => {
                    //debug!("maybe making handle for {node:?} {name}");
                    if shape.len() >= 2 {
                        match output_data.get(name) {
                            Some(port) => {
                                // debug!("making image handle for {node:?} {name}");
                                let data = match port {
                                    PortData::Array(ForayArray::Float(a)) => {
                                        &Array3::<f64>::from_shape_vec(
                                            (a.shape()[0], a.shape()[1], 3),
                                            //    .strides((
                                            //    a.strides()[0] as usize,
                                            //    a.strides()[1] as usize,
                                            //    0,
                                            //)),
                                            a.indexed_iter()
                                                .flat_map(|(_, v)| [*v, *v, *v])
                                                .collect::<Vec<_>>(),
                                        )
                                        .expect("square matrix")
                                    }
                                    //PortData::ArrayComplex(a) => &Array3::<f64>::from_shape_vec(
                                    //    (
                                    //        (a.len() as f32).sqrt() as usize,
                                    //        (a.len() as f32).sqrt() as usize,
                                    //        3,
                                    //    ),
                                    //    a.iter()
                                    //        .map(|v| v.norm_sqr().sqrt())
                                    //        .flat_map(|v| [v, v, v])
                                    //        .collect::<Vec<_>>(),
                                    //)
                                    //.expect("square matrix"),
                                    _ => panic!("unsuported plot types {:?}", port),
                                };
                                Some(create_rgb_handle(data))
                            }
                            _ => None, //(None, PortData::ArrayReal(Default::default())),
                        }
                    } else {
                        None
                    }
                }
                _ => None,
            },
            _ => None,
        };
        Self { image_handle }
    }

    pub fn clear_hanlde(&mut self) {
        self.image_handle = None;
    }
}

use ndarray::Array3;

use super::ForayNodeInstance;

//fn create_grayscale_handle(data: &Array3<f64>) -> Handle {}
fn create_rgb_handle(data: &Array3<f64>) -> Handle {
    // trace!("Creating image handle for plot2d, {:?}", data.shape());
    let max = data.iter().fold(-f64::INFINITY, |a, &b| a.max(b));
    let min = data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let brightness = |p: f64| {
        let p = ((p - min) / (max - min)) as f32;
        let p = if p.is_nan() { 0.0 } else { p };
        (p * 255.0).round() as u8
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
