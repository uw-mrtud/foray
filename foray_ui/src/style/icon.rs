use iced::{
    widget::{text, Text},
    Alignment::Center,
};

use crate::SYMBOL_FONT;

pub fn icon<'a, T>(character: T) -> Text<'a>
where
    T: iced::advanced::text::IntoFragment<'a>,
{
    text(character)
        .font(SYMBOL_FONT)
        .width(14.0)
        .height(16.0)
        .align_y(Center)
}
