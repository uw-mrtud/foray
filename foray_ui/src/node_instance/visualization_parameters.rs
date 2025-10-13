use derive_more::Display;
use iced::{Alignment::Center, Element, Length, Rectangle};

use itertools::Itertools;
use ndarray::{ArrayD, ArrayViewD, Slice};
use serde::{Deserialize, Serialize};
use strum::VariantArray;

use crate::{
    node_instance::histogram::{Histogram, HistogramWidget},
    workspace::WorkspaceMessage,
};
use iced::widget::{column, *};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct VisualizationParameters {
    // #[serde(skip)]
    // pub array_shape: Vec<usize>,
    pub ndim_mapping: Vec<(DimMapping, usize)>,
    value_mapping: ValueMapping, //(f64, f64),
    #[serde(skip)]
    pub histogram: Option<Histogram>,
    pub complex_map: RIMP,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct ValueMapping {
    floor: f32,
    ceil: f32,
}

impl Default for ValueMapping {
    fn default() -> Self {
        Self {
            floor: 0.0,
            ceil: 1.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum DimMapping {
    X,
    Y,
    Slice(usize),
}
impl DimMapping {
    fn matches(&self, selected: DimMapping) -> bool {
        match self {
            DimMapping::X => match selected {
                DimMapping::X => true,
                _ => false,
            },
            DimMapping::Y => match selected {
                DimMapping::Y => true,
                _ => false,
            },
            DimMapping::Slice(_) => match selected {
                DimMapping::Slice(_) => true,
                _ => false,
            },
        }
    }

    fn clear(&self) -> Self {
        match self {
            DimMapping::X => DimMapping::X,
            DimMapping::Y => DimMapping::Y,
            DimMapping::Slice(_) => DimMapping::Slice(0),
        }
    }
}

impl std::fmt::Display for DimMapping {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DimMapping::X => write!(f, "X"),
            DimMapping::Y => write!(f, "Y"),
            DimMapping::Slice(_) => write!(f, "Slice"),
        }
    }
}
#[derive(Clone, Copy, Debug, Display, PartialEq, Default, VariantArray, Serialize, Deserialize)]
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
pub fn default_dim_mapping(dims: &Vec<usize>) -> Vec<(DimMapping, usize)> {
    let mut mapping = Vec::new();
    if dims.len() > 0 {
        // dims[0] = number of rows = Y
        mapping.push((DimMapping::Y, dims[0]))
    }
    if dims.len() > 1 {
        // dims[x] = number of columns = X
        mapping.push((DimMapping::X, dims[1]))
    }
    if dims.len() > 2 {
        for i in 2..dims.len() {
            mapping.push((DimMapping::Slice(0), dims[i]))
        }
    }
    mapping
}

impl VisualizationParameters {
    /// Update ndim mapping when the shape of the data might have changed
    pub(crate) fn update_dimension_lengths(&mut self, dimensions: Vec<usize>) {
        //totally new shape, reset mapping
        if self.ndim_mapping.len() != dimensions.len() {
            self.ndim_mapping = default_dim_mapping(&dimensions);
        // make sure lengths are up to date
        } else {
            self.ndim_mapping.iter_mut().zip(dimensions).for_each(
                |((old_dim, old_length), new_length)| {
                    *old_length = new_length;
                    match old_dim {
                        DimMapping::Slice(old_slice) => {
                            *old_slice = (*old_slice).clamp(0, new_length.saturating_sub(1));
                        }
                        _ => {}
                    };
                },
            );
        }
    }

    pub fn slice_array_2d<'a, T>(&self, a: &'a ArrayD<T>) -> ArrayViewD<'a, T> {
        let mut out =
            a.slice_each_axis(
                |axis_desc| match self.ndim_mapping[axis_desc.axis.index()].0 {
                    DimMapping::X | DimMapping::Y => Slice::from(..),
                    DimMapping::Slice(slice_index) => {
                        Slice::new(slice_index as isize, Some(slice_index as isize + 1), 1)
                    }
                },
            );

        let x_pos = self
            .ndim_mapping
            .iter()
            .find_position(|(mapping, _)| *mapping == DimMapping::X)
            .map(|(pos, _)| pos)
            .unwrap_or(0);
        let y_pos = self
            .ndim_mapping
            .iter()
            .find_position(|(mapping, _)| *mapping == DimMapping::Y)
            .map(|(pos, _)| pos)
            .unwrap_or(0);

        // Transpose if X comes before Y
        if x_pos < y_pos {
            out.swap_axes(x_pos, y_pos);
        }
        out
    }
    pub(crate) fn xy_length(&self) -> (usize, usize) {
        let x_len = self
            .ndim_mapping
            .iter()
            .find_map(|(m, len)| match *m == DimMapping::X {
                true => Some(*len),
                false => None,
            })
            .unwrap_or(1);
        let y_len = self
            .ndim_mapping
            .iter()
            .find_map(|(m, len)| match *m == DimMapping::Y {
                true => Some(*len),
                false => None,
            })
            .unwrap_or(1);
        (x_len, y_len)
    }

    pub(crate) fn image_bounds(&self, max_length: f32) -> Rectangle {
        let (len_x, len_y) = self.xy_length();

        let scale_factor = max_length / (len_x.max(len_y) as f32);

        Rectangle::new(
            (0.0, 0.0).into(),
            (len_x as f32 * scale_factor, len_y as f32 * scale_factor).into(),
        )
    }

    pub(crate) fn view<'a>(&'a self, node_id: u32) -> Element<'a, WorkspaceMessage> {
        let pick_list_size = 12.0;

        let rimp_widget = pick_list(RIMP::VARIANTS, Some(self.complex_map), move |value| {
            WorkspaceMessage::UpdateVisualization(
                node_id,
                VisualizationParameters {
                    complex_map: value,
                    ..self.clone()
                },
            )
        })
        .text_size(pick_list_size);

        // Slice
        let size_dim_display = text(format!(
            "[{}]",
            self.ndim_mapping
                .iter()
                .map(|(_, len)| len.to_string())
                .join(",")
        ));
        let slice_dim_display = text(format!(
            "[{}]",
            self.ndim_mapping
                .iter()
                .map(|(dim, _len)| match dim {
                    DimMapping::X => " X: ".to_string(),
                    DimMapping::Y => " Y: ".to_string(),
                    DimMapping::Slice(s) => s.to_string(),
                })
                .join(",")
        ));
        let dim_mapping_options = [DimMapping::X, DimMapping::Y, DimMapping::Slice(0)];
        let slice_widget: Element<'_, WorkspaceMessage> = column(
            self.ndim_mapping
                .iter()
                .enumerate()
                .map(|(i, (current_mapping, len))| {
                    // If X, or Y is selected, we swap the current row and the existing X/Y row
                    let pick_list_message = move |selected| {
                        let mut new_parameters = self.clone();

                        // location of first pre-existing value matching selection
                        let replacement_index = new_parameters
                            .ndim_mapping
                            .iter()
                            .find_position(|(m, _len)| m.matches(selected));

                        if let Some((ri, _)) = replacement_index {
                            new_parameters.ndim_mapping[ri].0 = current_mapping.clear();
                        }
                        new_parameters.ndim_mapping[i].0 = selected;

                        WorkspaceMessage::UpdateVisualization(node_id, new_parameters)
                    };
                    row![
                        text(i),
                        match current_mapping {
                            DimMapping::X | DimMapping::Y => Element::<'_, WorkspaceMessage>::from(
                                pick_list(
                                    dim_mapping_options,
                                    Some(current_mapping),
                                    pick_list_message,
                                )
                                .text_size(pick_list_size)
                            ),
                            DimMapping::Slice(slice) => row![
                                pick_list(
                                    dim_mapping_options,
                                    Some(current_mapping),
                                    pick_list_message,
                                )
                                .text_size(pick_list_size),
                                text(slice).width(25),
                                slider(0.0..=(len - 1) as f32, *slice as f32, move |value| {
                                    let mut new_parameters = self.clone();
                                    new_parameters.ndim_mapping[i].0 =
                                        DimMapping::Slice(value as usize);
                                    WorkspaceMessage::UpdateVisualization(node_id, new_parameters)
                                })
                            ]
                            .align_y(Center)
                            .spacing(2.0)
                            .into(),
                        }
                    ]
                    .align_y(Center)
                    .spacing(4.0)
                    .into()
                }),
        )
        .spacing(2.0)
        .into();

        // Value mapping

        let value_mapping_widgets: Element<'_, WorkspaceMessage> = match &self.histogram {
            Some(histogram) => {
                let histogram_view = canvas(HistogramWidget::new(
                    histogram,
                    self.value_mapping.floor,
                    self.value_mapping.ceil,
                ));
                let floor_row = row![
                    text("floor").width(40),
                    slider(
                        histogram.min..=histogram.max,
                        self.value_mapping.floor,
                        move |value| {
                            let mut new_parameters = self.clone();
                            new_parameters.value_mapping.floor = value;
                            WorkspaceMessage::UpdateVisualization(node_id, new_parameters)
                        }
                    )
                    .step(0.1)
                ]
                .align_y(Center);

                let ceil_row = row![
                    text("ceil").width(40),
                    slider(
                        histogram.min..=histogram.max,
                        self.value_mapping.ceil,
                        move |value| {
                            let mut new_parameters = self.clone();
                            new_parameters.value_mapping.ceil = value;
                            WorkspaceMessage::UpdateVisualization(node_id, new_parameters)
                        }
                    )
                    .step(0.1)
                ]
                .align_y(Center);
                column![
                    row!["Value Mapping"],
                    horizontal_rule(1.0),
                    container(histogram_view.width(Length::Fill).height(50.0)).padding(6.0),
                    floor_row,
                    ceil_row
                ]
                .into()
            }
            None => column![].into(),
        };

        column![
            row![rimp_widget].align_y(Center),
            vertical_space().height(10.0),
            row!["Dimensions"],
            horizontal_rule(1.0),
            row!["Size: ", size_dim_display],
            row!["Slice: ", slice_dim_display],
            slice_widget,
            vertical_space().height(10.0),
            value_mapping_widgets
        ]
        .spacing(2.0)
        .into()
    }

    pub(crate) fn map_value(&self, x: f64) -> f64 {
        let (floor, ceil) = (
            self.value_mapping.floor as f64,
            self.value_mapping.ceil as f64,
        );

        let m = 1.0 / (ceil - floor);
        let b = floor / (floor - ceil);
        let y = m * x + b;
        y.clamp(0.0, 1.0)
    }
}
