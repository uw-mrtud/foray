use foray_data_model::node::{ForayArray, PortData};

use iced::{
    widget::canvas::{self, Program},
    Renderer, Theme,
};
use serde::{Deserialize, Serialize};

use crate::{node_instance::value_mapping::ValueMapping, workspace::WorkspaceMessage};

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct Histogram {
    pub min: f32,
    pub max: f32,
    pub counts: Vec<usize>,
}

impl Histogram {
    pub fn new(port_data: &PortData, value_mapping: &ValueMapping) -> Option<Self> {
        let n_bins = 100;
        match port_data {
            PortData::Array(foray_array) => match foray_array {
                ForayArray::Integer(array) => {
                    let max = array.iter().max().unwrap_or(&1);
                    let min = array.iter().min().unwrap_or(&0);
                    let mut counts = vec![0; n_bins];
                    array.iter().for_each(|v| {
                        let bin = ((v - min) / (max - min)) as usize * n_bins;
                        counts[bin.clamp(0, n_bins - 1)] += 1;
                    });
                    Some(Self {
                        min: *min as f32,
                        max: *max as f32,
                        counts,
                    })
                }
                ForayArray::Float(array) => {
                    let max = *array
                        .iter()
                        .max_by(|a: &&f64, b: &&f64| a.total_cmp(b))
                        .unwrap_or(&1.0);
                    let min = *array
                        .iter()
                        .min_by(|a: &&f64, b: &&f64| a.total_cmp(b))
                        .unwrap_or(&0.0);
                    let mut counts = vec![0; n_bins];
                    array.iter().for_each(|v| {
                        let bin = (((v - min) / (max - min)) * n_bins as f64) as usize;
                        counts[bin.clamp(0, n_bins - 1)] += 1;
                    });
                    Some(Self {
                        min: min as f32,
                        max: max as f32,
                        counts,
                    })
                }
                ForayArray::Complex(array) => {
                    let mapped_array: Vec<_> = array
                        .iter()
                        .map(|c| value_mapping.value_map_complex(*c))
                        .collect();

                    let max = mapped_array
                        .iter()
                        .max_by(|a: &&f64, b: &&f64| a.total_cmp(b))
                        .unwrap_or(&1.0);
                    let min = mapped_array
                        .iter()
                        .min_by(|a: &&f64, b: &&f64| a.total_cmp(b))
                        .unwrap_or(&0.0);

                    let mut counts = vec![0; n_bins];
                    mapped_array.iter().for_each(|v| {
                        let bin = (((v - min) / (max - min)) * n_bins as f64) as usize;
                        counts[bin.clamp(0, n_bins - 1)] += 1;
                    });
                    Some(Self {
                        min: *min as f32,
                        max: *max as f32,
                        counts,
                    })
                }
                ForayArray::Boolean(_) => None,
                ForayArray::String(_) => None,
                ForayArray::Object(_) => None,
            },
            _ => None,
        }
    }
}

impl Program<WorkspaceMessage> for ValueMapping {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: iced::Rectangle,
        _cursor: iced::advanced::mouse::Cursor,
    ) -> Vec<canvas::Geometry<Renderer>> {
        let histogram = if let Some(histogram) = &self.histogram {
            histogram
        } else {
            return vec![];
        };

        let mut frame = canvas::Frame::new(renderer, bounds.size());

        let color_bar_height = 15.0;

        let hist_height = bounds.height - color_bar_height;
        let hist_width = bounds.width - 1.0;

        let bin_spacing = 0.0;
        let bin_width = (hist_width) / histogram.counts.len() as f32;

        let max_count = histogram.counts.iter().max().unwrap_or(&1);

        // Draw bar graph
        for (i, count) in histogram.counts.iter().enumerate() {
            let x = i as f32 * (bin_width + bin_spacing);
            if *count > 0 {
                let hist_bar_height = ((*count as f32 / *max_count as f32) * hist_height).max(1.0);
                let y = hist_height - hist_bar_height;

                frame.fill_rectangle(
                    (x, y).into(),
                    (bin_width, hist_bar_height).into(),
                    theme.palette().text,
                );
            }
        }

        // map a value in the histograms distribution between 0.0 and 1.0
        let hist_to_norm = |v: f32| (v - histogram.min) / (histogram.max - histogram.min);

        // map a value between 0.0 and 1.0 to histogram's range
        let norm_to_hist = |v: f32| ((histogram.max - histogram.min) * v) + histogram.min;

        // Draw floor/ceil markers
        let floor_x = hist_width * hist_to_norm(self.floor);
        frame.fill_rectangle(
            (floor_x as f32, 0.0).into(),
            (1.0, hist_height).into(),
            theme.palette().primary,
        );
        let ceil_x = hist_width * hist_to_norm(self.ceil);
        frame.fill_rectangle(
            (ceil_x as f32, 0.0).into(),
            (1.0, hist_height).into(),
            theme.palette().primary,
        );

        // Draw color bar
        for x in 0..bounds.width as usize {
            let hist_value = norm_to_hist(x as f32 / bounds.width);
            let [r, g, b, _] = self.color_map_real(hist_value as f64);
            frame.fill_rectangle(
                (x as f32, hist_height).into(),
                (1.0, color_bar_height).into(),
                iced::Color::from_rgb8(r, g, b),
            );
        }

        vec![frame.into_geometry()]
    }
}
