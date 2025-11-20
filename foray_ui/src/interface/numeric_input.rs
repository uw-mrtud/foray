use iced::{
    alignment::Horizontal::Right,
    widget::{column, *},
    Alignment::Center,
    Color, Element,
    Length::{Fill, Shrink},
};

#[derive(Clone, PartialEq, Debug, Default, PartialOrd)]
pub enum PartialUIValue {
    #[default]
    Complete,
    Parsable(String),
    UnParsable(String),
}

/// A numeric text input box that will parse values, indicating when an invalid number is entered,
/// and does not update the true value until the error is corrected
///
/// This is complicated by intermediate strings that are encountered as the user enters a
/// value, for example -0 cannot be immediatley saved in the data model as `0` because then "0"
/// would be dispayed to the user, which would be frustring if trying to enter -0.1
pub fn numeric_input<'a, F, M>(
    value: f32,
    in_progress_widget_string: PartialUIValue,
    update_message: F,
) -> Element<'a, M>
where
    F: Fn(f32, PartialUIValue) -> M + 'a,
    M: std::clone::Clone + 'a,
{
    column![text_input(
        "n/a",
        &match &in_progress_widget_string {
            PartialUIValue::Complete => format!("{}", fp_round(value)).to_string(),
            PartialUIValue::Parsable(s) => s.clone(),
            PartialUIValue::UnParsable(s) => s.clone(),
        }
    )
    .on_input(
        move |new_value: String| if let Ok(parsed) = new_value.parse::<f32>() {
            if parsed.to_string() == new_value {
                update_message(parsed, PartialUIValue::Complete)
            } else {
                update_message(parsed, PartialUIValue::Parsable(new_value))
            }
        } else {
            update_message(value, PartialUIValue::UnParsable(new_value))
        }
    )
    .align_x(Right)
    .padding([1.0, 3.0])]
    .height(Shrink)
    .width(Fill)
    .align_x(Center)
    .into()
}

pub fn styled_text_input<'a, M: Clone + 'a>(input: TextInput<'a, M>) -> Element<'a, M> {
    column![
        input
            .padding(0)
            .style(|t, s| {
                let d = text_input::default(t, s);
                text_input::Style {
                    border: d.border.color(Color::TRANSPARENT),
                    background: iced::Background::Color(Color::TRANSPARENT),
                    ..d
                }
            })
            .align_x(Center),
        container(rule::horizontal(0)).padding(4.).height(1),
    ]
    .height(Shrink)
    .width(Fill)
    .align_x(Center)
    .into()
}

fn fp_round(x: f32) -> f32 {
    let y = 10i32.pow(6) as f32;
    (x * y).round() / y
}
