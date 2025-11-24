use crate::series_vis::{AxisOptions, Extent, SeriesVis, SeriesVisOptions};

use iced::{
    Alignment::Center,
    Element,
    widget::{column, row, space, text_input, toggler},
};
use std::ops::Range;

use crate::layout::section;
const ROW_SPACING: f32 = 8.0;
const COL_SPACING: f32 = 4.0;

impl AxisOptions {
    fn view<'a, T: Clone + 'a, U: 'a>(&'a self, range: Range<f64>, update: U) -> Element<'a, T>
    where
        U: Fn(AxisOptions) -> T + Clone,
    {
        let update_1 = update.clone();
        let update_2 = update.clone();
        let update_3 = update.clone();
        let update_4 = update.clone();
        let update_5 = update.clone();

        let (extent_min, extent_max, is_auto) = match self.extent {
            Extent::Auto => (range.start, range.end, true),
            Extent::Fixed(min, max) => (min, max, false),
        };

        column![
            //// Range
            row![
                "Range",
                space::horizontal().width(ROW_SPACING),
                column![
                    row![
                        //// Auto
                        "Auto",
                        toggler(is_auto).on_toggle(move |val| {
                            let mut new_axis_options = self.clone();

                            match val {
                                true => new_axis_options.extent = Extent::Auto,
                                false => {
                                    new_axis_options.extent = Extent::Fixed(extent_min, extent_max)
                                }
                            }

                            update_3(new_axis_options)
                        }),
                    ]
                    .spacing(ROW_SPACING),
                    if !is_auto {
                        row![
                            //// Min
                            text_input("...", &extent_min.to_string()).on_input(move |val| {
                                let mut new_axis_options = self.clone();
                                new_axis_options.extent =
                                    Extent::Fixed(val.parse().unwrap_or_default(), extent_max);
                                update_4(new_axis_options)
                            }),
                            //// Max
                            text_input("...", &extent_max.to_string()).on_input(move |val| {
                                let mut new_axis_options = self.clone();
                                new_axis_options.extent =
                                    Extent::Fixed(extent_min, val.parse().unwrap_or_default());
                                update_5(new_axis_options)
                            })
                        ]
                        .spacing(ROW_SPACING)
                    } else {
                        row![]
                    }
                ]
            ]
            .align_y(Center)
            .spacing(ROW_SPACING),
            //// Label
            row![
                "Label",
                text_input("...", &self.label.clone().unwrap_or_default()).on_input(move |val| {
                    let mut new_axis_options = self.clone();
                    new_axis_options.label = if val == "" { None } else { Some(val) };

                    update_1(new_axis_options)
                })
            ]
            .align_y(Center)
            .spacing(ROW_SPACING),
            //// Grid
            row![
                "Grid",
                toggler(self.grid).on_toggle(move |val| {
                    let mut new_axis_options = self.clone();
                    new_axis_options.grid = val;

                    update_2(new_axis_options)
                })
            ]
            .align_y(Center)
            .spacing(ROW_SPACING),
        ]
        .spacing(COL_SPACING)
        .into()
    }
}

impl SeriesVis {
    pub fn config_view<'a, T: Clone + 'a, U: 'a>(&'a self, update: U) -> Element<'a, T>
    where
        U: Fn(SeriesVisOptions) -> T + Clone,
    {
        //TODO: Avoid creating this array...
        //Store x_data in SeriesVis?
        let x_data: Vec<f64> = (0..(self.data.len())).map(|x| x as f64).collect();
        let x_range = self.vis_options.x.range(&x_data);

        let y_range = self
            .vis_options
            .y
            .range(&self.data.as_slice().unwrap_or_default());

        let update_1 = update.clone();
        let update_2 = update.clone();
        let update_3 = update.clone();
        column![
            //// Title
            row![
                "Title",
                text_input("...", &self.vis_options.title.clone().unwrap_or_default()).on_input(
                    move |val| {
                        let mut new_vis_options = self.vis_options.clone();
                        new_vis_options.title = if val == "" { None } else { Some(val) };

                        update_1(new_vis_options)
                    }
                )
            ]
            .align_y(Center)
            .spacing(4.0),
            section(
                "X",
                self.vis_options.x.view(x_range, move |updated_axis| {
                    let mut new_vis_options = self.vis_options.clone();
                    new_vis_options.x = updated_axis;
                    update_2(new_vis_options)
                })
            ),
            section(
                "Y",
                self.vis_options.y.view(y_range, move |updated_axis| {
                    let mut new_vis_options = self.vis_options.clone();
                    new_vis_options.y = updated_axis;
                    update_3(new_vis_options)
                })
            )
        ]
        .spacing(8.0)
        .into()
    }
}
