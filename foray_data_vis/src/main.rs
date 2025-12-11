use std::f64::consts::PI;

use derive_more::Display;
use foray_data_vis::{
    image_vis::ImageVis,
    series_vis::{SeriesVis, SeriesVisOptions},
    svg_vis::SvgVis,
};
use iced::{
    Alignment::Center,
    Element,
    Length::Fill,
    widget::{column, container, image, pick_list, row, space, svg},
};
use ndarray::{Array1, Array2};

fn main() {
    let _ = iced::application(|| State::default(), update, view).run();
}

#[derive(Debug, Clone)]
enum Message {
    UpdateSeriesVis(SeriesVisOptions),
    ChangeScreen(CurrentScreen),
}

#[derive(Debug, PartialEq, Clone, Copy, Display)]
enum CurrentScreen {
    Image,
    Series,
    Svg,
}

struct State {
    image_vis: ImageVis,
    series_vis: SeriesVis,
    svg_vis: SvgVis,
    screen: CurrentScreen,
}

impl Default for State {
    fn default() -> Self {
        let mut image_data = Array2::default((16, 16));
        image_data.iter_mut().enumerate().for_each(|(i, v)| {
            *v = i as f64;
        });

        let x_data: Array1<f64> = (0..100).map(|x| x as f64).collect();
        let y_data = vec![
            x_data.iter().map(|x| (x / 10.0).sin()).collect(),
            x_data.iter().map(|x| (x / 10.0 - PI / 6.0).sin()).collect(),
            x_data.iter().map(|x| (x / 10.0 - PI / 3.0).sin()).collect(),
        ];

        Self {
            image_vis: ImageVis::new(image_data),
            series_vis: SeriesVis::new(
                x_data,
                y_data,
                SeriesVisOptions::new(Some("sin(x)".to_string())),
            ),
            svg_vis: SvgVis::new(
                r##"
<svg height="400" width="450" xmlns="http://www.w3.org/2000/svg">

<!-- Draw the paths -->
  <path id="lineAB" d="M 100 350 l 150 -300" stroke="red" stroke-width="4"/>
  <path id="lineBC" d="M 250 50 l 150 300" stroke="red" stroke-width="4"/>
  <path id="lineMID" d="M 175 200 l 150 0" stroke="green" stroke-width="4"/>
  <path id="lineAC" d="M 100 350 q 150 -300 300 0" stroke="blue" fill="none" stroke-width="4"/>

<!-- Mark relevant points -->
  <g stroke="black" stroke-width="3" fill="black">
    <circle id="pointA" cx="100" cy="350" r="4" />
    <circle id="pointB" cx="250" cy="50" r="4" />
    <circle id="pointC" cx="400" cy="350" r="4" />
  </g>

<!-- Label the points -->
  <g font-size="30" font-family="sans-serif" fill="green" text-anchor="middle">
    <text x="100" y="350" dx="-30">A</text>
    <text x="250" y="50" dy="-10">B</text>
    <text x="400" y="350" dx="30">C</text>
  </g>
  
Sorry, your browser does not support inline SVG.
</svg>"##,
            ),
            screen: CurrentScreen::Series,
        }
    }
}

fn update(state: &mut State, message: Message) {
    match message {
        Message::UpdateSeriesVis(updated_options) => {
            state.series_vis.update_options(updated_options)
        }
        Message::ChangeScreen(current_screen) => state.screen = current_screen,
    }
}

fn view(state: &'_ State) -> Element<'_, Message> {
    let vis_size = 500;
    column![
        pick_list(
            [
                CurrentScreen::Series,
                CurrentScreen::Image,
                CurrentScreen::Svg
            ],
            Some(state.screen),
            |v| Message::ChangeScreen(v)
        ),
        match state.screen {
            CurrentScreen::Image => row![
                image(&state.image_vis.image_handle)
                    .width(vis_size)
                    .height(vis_size),
                space::vertical(),
            ]
            .height(Fill)
            .align_y(Center),
            CurrentScreen::Series => row![
                container(state.series_vis.config_view(Message::UpdateSeriesVis)).width(200),
                svg(state.series_vis.svg().handle.clone())
                    // .content_fit(iced::ContentFit::None)
                    .width(vis_size)
                    .height(vis_size)
            ]
            .height(Fill)
            .align_y(Center),
            CurrentScreen::Svg => row![
                svg(state.svg_vis.svg().handle.clone())
                    // .content_fit(iced::ContentFit::None)
                    .width(vis_size)
                    .height(vis_size)
            ]
            .height(Fill)
            .align_y(Center),
        },
    ]
    .width(Fill)
    .align_x(Center)
    .padding(10)
    .into()
}
