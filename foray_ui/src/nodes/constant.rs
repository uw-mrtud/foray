use crate::{app::Message, nodes::UINodeTemplate};
use iced::{
    widget::{column, container, slider, text},
    Alignment::Center,
    Element,
    Length::Fill,
};

use super::RustNode;

pub fn view<'a>(id: u32, value: f64) -> Element<'a, Message> {
    container(
        column![
            text(format!("{value:.1}")),
            slider(0.0..=2.0, value, move |value| {
                Message::UpdateNodeTemplate(id, UINodeTemplate::RustNode(RustNode::Constant(value)))
            })
            .step(0.05)
            .width(Fill),
        ]
        .align_x(Center)
        .padding([0., 10.]),
    )
    .center_y(Fill)
    .align_right(Fill)
    .into()
}
