use derive_more::derive::{Debug, Display};
use iced::Element;
use iced::{widget::*, Alignment::Center};
use serde::{Deserialize, Serialize};
use strum::{EnumString, VariantNames};

use crate::{
    app::Message,
    interface::numeric_input::{self, PartialUIValue},
    StableMap,
};
pub type NodeUIParameters = StableMap<String, NodeUIWidget>;

#[derive(
    Clone, Debug, Display, EnumString, VariantNames, Serialize, Deserialize, PartialEq, PartialOrd,
)]
pub enum NodeUIWidget {
    #[display("{_0}")]
    Slider(f32, #[serde(skip)] PartialUIValue),
    #[display("{_0}")]
    NumberField(f32, #[serde(skip)] PartialUIValue),
    CheckBox(bool),
}

impl NodeUIWidget {
    pub fn view<'a, F>(&'a self, update_message: F) -> Element<'a, Message>
    where
        F: Fn(NodeUIWidget) -> Message + Clone + 'a,
    {
        // Needs 2 of these for borrow checker (is there a cleaner way?)
        let update_message_2 = update_message.clone();
        match self {
            NodeUIWidget::Slider(v, in_progress) => row![
                row![numeric_input::numeric_input(
                    *v,
                    in_progress.clone(),
                    move |new_v, in_progress: PartialUIValue| {
                        update_message(Self::Slider(new_v, in_progress))
                    },
                )]
                .width(60.0),
                slider(-1.0..=1.0, *v, move |new_v| {
                    update_message_2(Self::Slider(new_v, PartialUIValue::Complete))
                })
                .step(0.01)
            ]
            .align_y(Center)
            .spacing(4.0)
            .into(),
            NodeUIWidget::NumberField(v, in_progress) => row![
                horizontal_space(),
                row![numeric_input::numeric_input(
                    *v,
                    in_progress.clone(),
                    move |new_v, in_progress: PartialUIValue| {
                        update_message(Self::NumberField(new_v, in_progress))
                    },
                )]
                .width(60.0)
            ]
            .align_y(Center)
            .into(),
            NodeUIWidget::CheckBox(_v) => todo!(),
        }
    }
}
