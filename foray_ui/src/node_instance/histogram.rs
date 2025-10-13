use foray_data_model::node::{ForayArray, PortData};

use iced::{
    widget::canvas::{self, Program},
    Renderer, Theme,
};
use serde::{Deserialize, Serialize};

use crate::workspace::WorkspaceMessage;

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct Histogram {
    pub min: f32,
    pub max: f32,
    pub counts: Vec<usize>,
}

impl Histogram {
    pub fn new(port_data: &PortData) -> Option<Self> {
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
                    let max = array
                        .iter()
                        .map(|v| v.norm())
                        .max_by(|a: &f64, b: &f64| a.total_cmp(b))
                        .unwrap_or(1.0);
                    let min = array
                        .iter()
                        .map(|v| v.norm())
                        .min_by(|a: &f64, b: &f64| a.total_cmp(b))
                        .unwrap_or(0.0);

                    let mut counts = vec![0; n_bins];
                    array.iter().for_each(|c| {
                        let v = c.norm();
                        let bin = (((v - min) / (max - min)) * n_bins as f64) as usize;
                        counts[bin.clamp(0, n_bins - 1)] += 1;
                    });
                    Some(Self {
                        min: min as f32,
                        max: max as f32,
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

// Short lived struct just used for drawing the histogram
pub(super) struct HistogramWidget<'a> {
    histogram: &'a Histogram,
    floor: f32,
    ceil: f32,
}

impl<'a> HistogramWidget<'a> {
    pub(super) fn new(histogram: &'a Histogram, floor: f32, ceil: f32) -> Self {
        Self {
            histogram,
            floor,
            ceil,
        }
    }
}

impl Program<WorkspaceMessage> for HistogramWidget<'_> {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: iced::Rectangle,
        _cursor: iced::advanced::mouse::Cursor,
    ) -> Vec<canvas::Geometry<Renderer>> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        let height = bounds.height;
        let width = bounds.width - 1.0;

        let bin_spacing = 0.0;
        let bin_width = (width) / self.histogram.counts.len() as f32;

        let max_count = self.histogram.counts.iter().max().unwrap_or(&1);

        // Draw bar graph
        for (i, count) in self.histogram.counts.iter().enumerate() {
            let x = i as f32 * (bin_width + bin_spacing);
            if *count > 0 {
                let bar_height = ((*count as f32 / *max_count as f32) * height).max(1.0);
                let y = height - bar_height;

                frame.fill_rectangle(
                    (x, y).into(),
                    (bin_width, bar_height).into(),
                    theme.palette().text,
                );
            }
        }

        // Draw floor/ceil markers
        let floor_x =
            width * (self.floor - self.histogram.min) / (self.histogram.max - self.histogram.min);
        frame.fill_rectangle(
            (floor_x as f32, 0.0).into(),
            (1.0, height).into(),
            theme.palette().primary,
        );
        let ceil_x =
            width * (self.ceil - self.histogram.min) / (self.histogram.max - self.histogram.min);
        frame.fill_rectangle(
            (ceil_x as f32, 0.0).into(),
            (1.0, height).into(),
            theme.palette().primary,
        );

        //TODO: draw color bar legend

        vec![frame.into_geometry()]
    }
}
