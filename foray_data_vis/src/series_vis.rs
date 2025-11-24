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

#[derive(Default, Debug, Clone)]
pub enum Extent {
    #[default]
    Auto,
    Fixed(f64, f64),
}

#[derive(Default, Debug, Clone)]
pub struct AxisOptions {
    pub(crate) extent: Extent,
    pub(crate) label: Option<String>,
    pub(crate) grid: bool,
}
impl AxisOptions {
    pub(crate) fn range(&self, data: &[f64]) -> Range<f64> {
        match self.extent {
            Extent::Auto => {
                let min = data.iter().copied().reduce(f64::min).unwrap_or_default();
                let max = data.iter().copied().reduce(f64::max).unwrap_or_default();
                let extent = max - min;
                let padding = extent * 0.0; //0.01;

                (min - padding)..(max + padding)
            }
            Extent::Fixed(min, max) => min..max,
        }
    }
}

#[derive(Default, Debug, Clone)]
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

pub struct SeriesVis {
    pub(crate) data: Array1<f64>,
    pub(crate) svg: Svg,
    pub(crate) vis_options: SeriesVisOptions,
}

impl SeriesVis {
    pub fn new(data: Array1<f64>, vis_options: SeriesVisOptions) -> Self {
        Self {
            svg: Self::compute_svg(&data, &vis_options),
            data,
            vis_options,
        }
    }
    pub fn update_options(&mut self, options: SeriesVisOptions) {
        self.vis_options = options;
        self.svg = Self::compute_svg(&self.data, &self.vis_options)
    }

    pub fn svg(&self) -> &Svg {
        &self.svg
    }
    fn compute_svg(y_data: &Array1<f64>, vis_options: &SeriesVisOptions) -> Svg {
        let x_data: Vec<f64> = (0..(y_data.len())).map(|x| x as f64).collect();
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

            let series_color = RGBColor(250, 120, 120);
            let label_style = ("sans-serif", 15 * scale, &fg_color);
            let title_style = ("sans-serif", 24 * scale, &fg_color);

            let x_range = vis_options.x.range(&x_data);
            let y_range = vis_options.y.range(y_data.as_slice().unwrap_or_default());

            //// Chart Context
            let mut chart = {
                let mut partial = ChartBuilder::on(&root_drawing_area);

                if let Some(title) = &vis_options.title {
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

                if !vis_options.x.grid {
                    mesh_style.disable_x_mesh();
                };

                if !vis_options.y.grid {
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
                    .x_desc(vis_options.x.label.clone().unwrap_or_default())
                    .y_desc(vis_options.y.label.clone().unwrap_or_default())
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
            let data_coord = x_data.iter().zip(y_data).map(|(x, y)| (*x, *y));

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
                        .draw_series(LineSeries::new(
                            segment,
                            ShapeStyle {
                                color: series_color.into(),
                                filled: false,
                                stroke_width: 2 * scale,
                            },
                        ))
                        .unwrap();
                });

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

            // chart
            //     .draw_series(LineSeries::new(
            //         x_data.iter().zip(y_data).map(|(x, y)| (*x, *y)),
            //         ShapeStyle {
            //             color: series_color.into(),
            //             filled: false,
            //             stroke_width: 20,
            //         },
            //     ))
            //     .unwrap();

            //// Chart Labels
            chart
                .configure_series_labels()
                .label_font(("sans-serif", 2 * scale, &WHITE))
                .draw()
                .unwrap();
        }

        iced::advanced::svg::Svg::new(iced::advanced::svg::Handle::from_memory(
            svg_buffer.into_bytes(),
        ))
    }
}
