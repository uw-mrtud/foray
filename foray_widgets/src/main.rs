use iced::{
    Alignment::{self, Center},
    Element,
    Length::{Fill, Shrink},
    Size,
    widget::{column, container, row, text, text_input},
};

fn main() {
    iced::application(|| 10.0, update, view)
        .window_size(Size::new(300.0, 300.0))
        .run()
        .unwrap();
}
#[derive(Debug, Clone)]
enum Message {
    UpdateValue(f32),
}

fn update(state: &mut f32, message: Message) {
    match message {
        Message::UpdateValue(val) => *state = val,
    }
}

fn view(state: &'_ f32) -> Element<'_, Message> {
    container(
        column![
            text(format_f32(*state)),
            row![
                "Standard text input:",
                text_input::TextInput::new("placeholder", &format_f32(*state))
                    .on_input(|s| Message::UpdateValue(s.parse().unwrap_or_default()))
                    .width(60)
            ]
            .align_y(Center)
            .spacing(4.0),
            row![
                "Foray numeric input:",
                foray_widgets::numeric_input::NumericInput::new("placeholder", &format_f32(*state))
                    .on_input(|s| Message::UpdateValue(s.parse().unwrap_or_default()))
                    .width(60)
            ]
            .align_y(Center)
            .spacing(4.0)
        ]
        .spacing(4.0)
        .width(Shrink)
        .align_x(Alignment::End),
    )
    .center(Fill)
    .into()
}

fn format_f32(val: f32) -> String {
    format!("{:.2}", val)
}
