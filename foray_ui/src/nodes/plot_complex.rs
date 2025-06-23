use super::{PortData, RustNode};
use crate::app::Message;
use crate::gui_node::{PortDataContainer, PortDataReference};
use crate::interface::node::{INNER_NODE_WIDTH, NODE_BORDER_WIDTH};
use crate::math::Vector;
use crate::nodes::NodeTemplate;
use crate::StableMap;
use iced::widget::image::Handle;
use iced::widget::{button, container, horizontal_space, image, row, text, text_input};
use iced::Alignment::Center;
use iced::{widget::column, Element};
use log::trace;
use ndarray::Array3;
use serde::{Deserialize, Serialize};

// Rectanlge specified by center position, width and height
// y is up
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct Rect {
    pub center: Vector,
    pub width: f32,
    pub height: f32,
}
impl Rect {
    pub fn right(&self) -> f32 {
        self.center.x + self.width / 2.
    }
    pub fn left(&self) -> f32 {
        self.center.x - self.width / 2.
    }
    pub fn top(&self) -> f32 {
        self.center.y + self.height / 2.
    }
    pub fn bottom(&self) -> f32 {
        self.center.y - self.height / 2.
    }
}
impl Default for Rect {
    fn default() -> Self {
        Rect {
            center: [0.5, 0.5].into(),
            width: 1.,
            height: 1.,
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Clone)]
pub struct Plot2D {
    pub rect: Rect,
    #[serde(skip)]
    pub image_handle: Option<Handle>,
}

impl PartialOrd for Plot2D {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.rect.partial_cmp(&other.rect) {
            Some(ord) => Some(ord),
            ord => ord,
        }
    }
}

impl Plot2D {
    pub fn view<'a>(
        &self,
        _id: u32,
        _input_data: StableMap<String, PortDataContainer>,
    ) -> Element<'a, Message> {
        match &self.image_handle {
            Some(handle) => container(
                image(handle)
                    .filter_method(image::FilterMethod::Nearest)
                    .width(INNER_NODE_WIDTH * 2.)
                    .height(INNER_NODE_WIDTH * 2.),
            ),
            _ => container(text("")),
        }
        .padding(NODE_BORDER_WIDTH)
        .into()
    }

    pub fn config_view(
        &self,
        id: u32,
        _input_data: StableMap<String, PortDataContainer>,
    ) -> Option<Element<'_, Message>> {
        let center = self.rect.center;
        let width = self.rect.width;
        let height = self.rect.height;
        let message = move |rect| {
            Message::UpdateNodeTemplate(
                id,
                NodeTemplate::RustNode(RustNode::Plot2D(Self {
                    rect,
                    ..self.clone()
                })),
            )
        };
        let zoom_speed = 0.125;
        Some(
            column![
                row![
                    text("center:"),
                    horizontal_space(),
                    text("x"),
                    text_input("0", &center.x.to_string()).on_input(move |value| {
                        let mut n = self.rect;
                        n.center.x = value.parse().unwrap_or(0.);
                        message(n)
                    }),
                    text("y"),
                    text_input("0", &center.y.to_string()).on_input(move |value| {
                        let mut n = self.rect;
                        n.center.y = value.parse().unwrap_or(0.);
                        message(n)
                    }),
                ]
                .align_y(Center)
                .spacing(4.),
                row![
                    text("width:"),
                    horizontal_space(),
                    text_input("0", &width.to_string()).on_input(move |value| {
                        let mut n = self.rect;
                        n.width = value.parse().unwrap_or(1.0f32).max(0.001);
                        message(n)
                    }),
                ]
                .align_y(Center),
                row![
                    text("height:"),
                    horizontal_space(),
                    text_input("0", &height.to_string()).on_input(move |value| {
                        let mut n = self.rect;
                        n.height = value.parse().unwrap_or(1.0f32).max(0.001);
                        message(n)
                    }),
                ]
                .align_y(Center),
                row![
                    horizontal_space(),
                    button("+").on_press_with(move || {
                        let mut n = self.rect;
                        let aspect = n.width / n.height;
                        n.height -= zoom_speed;
                        n.height = n.height.max(0.01);
                        n.width -= zoom_speed * aspect;
                        n.width = n.width.max(0.1 * aspect);
                        message(n)
                    }),
                    button("-").on_press_with(move || {
                        let mut n = self.rect;
                        let aspect = n.width / n.height;
                        n.height += zoom_speed;
                        n.width += zoom_speed * aspect;
                        message(n)
                    }),
                ]
                .spacing(5.0)
                .align_y(Center),
                match &self.image_handle {
                    Some(handle) => container(
                        image(handle)
                            .filter_method(image::FilterMethod::Nearest)
                            .width(INNER_NODE_WIDTH * 2.)
                            .height(INNER_NODE_WIDTH * 2.),
                    ),
                    _ => container(text("")),
                }
            ]
            .spacing(5.0)
            .into(),
        )
    }

    fn create_image_handle(data: &Array3<f64>) -> Handle {
        trace!("Creating image handle for plot2d, {:?}", data.shape());
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

    pub(crate) fn input_changed(
        &mut self,
        input_data: StableMap<String, PortDataReference>,
    ) -> PortData {
        let (image_handle, port_data) = match input_data.get("a") {
            Some(port) => {
                let data = match &**port {
                    PortData::ArrayReal(a) => &Array3::<f64>::from_shape_vec(
                        (a.shape()[0], a.shape()[1], 3),
                        a.iter().flat_map(|v| [*v, *v, *v]).collect::<Vec<_>>(),
                    )
                    .expect("square matrix"),
                    PortData::ArrayComplex(a) => &Array3::<f64>::from_shape_vec(
                        (
                            (a.len() as f32).sqrt() as usize,
                            (a.len() as f32).sqrt() as usize,
                            3,
                        ),
                        a.iter()
                            .map(|v| v.norm_sqr().sqrt())
                            .flat_map(|v| [v, v, v])
                            .collect::<Vec<_>>(),
                    )
                    .expect("square matrix"),
                    _ => panic!("unsuported plot types {:?}", port),
                };
                (Some(Self::create_image_handle(data)), (**port).clone())
            }
            None => (None, PortData::ArrayReal(Default::default())),
        };
        self.image_handle = image_handle;
        port_data
    }
}
