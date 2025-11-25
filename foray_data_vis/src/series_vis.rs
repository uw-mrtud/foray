use std::ops::Range;

use iced::advanced::svg::Svg;
use itertools::Itertools;
use ndarray::Array1;
use plotters::{
    chart::ChartBuilder,
    prelude::IntoDrawingArea,
    series::LineSeries,
    style::{RGBAColor, RGBColor, ShapeStyle, WHITE},
};
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Extent {
    #[default]
    Auto,
    Fixed(f64, f64),
}

#[derive(Default, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AxisOptions {
    pub(crate) extent: Extent,
    pub(crate) label: Option<String>,
    pub(crate) grid: bool,
}
impl AxisOptions {
    pub(crate) fn range(&self, data_range: Range<f64>) -> Range<f64> {
        match self.extent {
            Extent::Auto => data_range,
            Extent::Fixed(min, max) => min..max,
        }
    }
}

#[derive(Default, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SeriesVisOptions {
    pub(crate) title: Option<String>,
    pub(crate) x: AxisOptions,
    pub(crate) y: AxisOptions,
}

impl SeriesVisOptions {
    pub fn new(title: Option<String>) -> Self {
        Self {
            title,
            x: Default::default(),
            y: Default::default(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SeriesVis {
    #[serde(skip)]
    pub(crate) x_data: Array1<f64>,
    #[serde(skip)]
    pub(crate) y_data: Vec<Array1<f64>>,
    #[serde(skip, default = "default_svg")]
    pub(crate) svg: Svg,
    pub(crate) vis_options: SeriesVisOptions,
}

fn default_svg() -> Svg {
    iced::advanced::svg::Svg::new(iced::advanced::svg::Handle::from_memory(&[]))
}

impl SeriesVis {
    pub fn new(
        x_data: Array1<f64>,
        y_data: Vec<Array1<f64>>,
        vis_options: SeriesVisOptions,
    ) -> Self {
        let mut s = Self {
            svg: iced::advanced::svg::Svg::new(iced::advanced::svg::Handle::from_memory(&[])),
            x_data,
            y_data,
            vis_options,
        };
        s.compute_svg();
        s
    }
    pub fn update_options(&mut self, options: SeriesVisOptions) {
        self.vis_options = options;
        self.compute_svg();
    }

    pub fn svg(&self) -> &Svg {
        &self.svg
    }

    pub fn vis_options(&self) -> &SeriesVisOptions {
        &self.vis_options
    }
    fn data_range<'a, T: Iterator<Item = &'a f64>>(data: T) -> std::ops::Range<f64> {
        let minmax = data.minmax();
        match minmax {
            itertools::MinMaxResult::NoElements => 0.0..0.0,
            itertools::MinMaxResult::OneElement(a) => *a..*a,
            itertools::MinMaxResult::MinMax(a, b) => *a..*b,
        }
    }

    pub(crate) fn x_data_range(&self) -> std::ops::Range<f64> {
        Self::data_range(self.x_data.iter())
    }
    pub(crate) fn y_data_range(&self) -> std::ops::Range<f64> {
        Self::data_range(self.y_data.iter().flatten())
    }

    fn compute_svg(&mut self) {
        let mut svg_buffer = String::new();
        {
            let scale = 20;
            let root_drawing_area = plotters::backend::SVGBackend::with_string(
                &mut svg_buffer,
                (300 * scale, 300 * scale),
            )
            .into_drawing_area();

            let fg_color = WHITE;
            let mesh_line_color = RGBAColor(70, 70, 70, 1.0);

            let series_palette = vec![
                RGBColor(250, 120, 120),
                RGBColor(120, 250, 120),
                RGBColor(120, 120, 250),
            ];

            let label_style = ("sans-serif", 15 * scale, &fg_color);
            let title_style = ("sans-serif", 24 * scale, &fg_color);

            let x_range = self.vis_options.x.range(self.x_data_range());
            let y_range = self.vis_options.y.range(self.y_data_range());

            //// Chart Context
            let mut chart = {
                let mut partial = ChartBuilder::on(&root_drawing_area);

                if let Some(title) = &self.vis_options.title {
                    partial.caption(title, title_style);
                }

                partial
                    .margin_top(15 * scale)
                    .margin_right(15 * scale)
                    .x_label_area_size(35 * scale)
                    .y_label_area_size(35 * scale)
                    .build_cartesian_2d(x_range.clone(), y_range.clone())
                    .unwrap()
            };

            //// Chart Mesh
            {
                let mut mesh_style = chart.configure_mesh();

                if !self.vis_options.x.grid {
                    mesh_style.disable_x_mesh();
                };

                if !self.vis_options.y.grid {
                    mesh_style.disable_y_mesh();
                };

                mesh_style
                    .label_style(label_style)
                    .max_light_lines(0)
                    .bold_line_style(ShapeStyle {
                        color: mesh_line_color,
                        filled: false,
                        stroke_width: 1 * scale,
                    })
                    .x_label_formatter(&|x| format!("{:.0}", x))
                    .x_desc(self.vis_options.x.label.clone().unwrap_or_default())
                    .y_desc(self.vis_options.y.label.clone().unwrap_or_default())
                    .set_all_tick_mark_size(2 * scale)
                    .axis_style(ShapeStyle {
                        color: fg_color.into(),
                        filled: true,
                        stroke_width: 1 * scale,
                    })
                    .draw()
                    .unwrap()
            };

            //// Chart Series
            for (i, y_series) in self.y_data.iter().enumerate() {
                let series_style = ShapeStyle {
                    color: series_palette[i % series_palette.len()].into(),
                    filled: false,
                    stroke_width: 2 * scale,
                };
                let data_coord = self.x_data.iter().zip(y_series).map(|(x, y)| (*x, *y));

                let delta = 0.0000000001;
                // break data into line segments that fall inside the bounding box
                data_coord
                    .chunk_by(move |(x, y)| {
                        (*x >= (x_range.start - delta))
                            && (*x <= (x_range.end + delta))
                            && (*y >= (y_range.start - delta))
                            && (*y <= (y_range.end + delta))
                    })
                    .into_iter()
                    .filter_map(|(io, segment)| match io {
                        true => Some(segment),
                        false => None,
                    })
                    .for_each(|segment| {
                        chart
                            .draw_series(LineSeries::new(segment, series_style))
                            .unwrap();
                    });
            }

            // let segments = data_coord
            //     .scan((false, 0), move |(past_io, segment), (x, y)| {
            //         let current_io = x_range.contains(&x) && y_range.contains(&y);
            //         //switching from out to in, or in to out
            //         if current_io != *past_io {
            //             *segment += 1;
            //         }
            //         *past_io = current_io;
            //         Some(((current_io, *segment), (x, y)))
            //     })
            //     .groupby();

            //// Chart Labels
            chart
                .configure_series_labels()
                .label_font(("sans-serif", 2 * scale, &WHITE))
                .draw()
                .unwrap();
        }

        self.svg = iced::advanced::svg::Svg::new(iced::advanced::svg::Handle::from_memory(
            svg_buffer.into_bytes(),
        ));
    }
}
