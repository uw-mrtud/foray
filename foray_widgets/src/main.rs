use iced::{
    Alignment::{self, Center},
    Element,
    Length::{Fill, Shrink},
    Size,
    widget::{button, column, container, row, text, text_input},
    window,
};

fn main() {
    iced::application(|| 10.0, update, view)
        .window_size(Size::new(350.0, 300.0))
        .subscription(|_| window::frames().map(|_| Message::Tick))
        .run()
        .unwrap();
}
#[derive(Debug, Clone)]
enum Message {
    UpdateValue(f32),
    Tick,
}

fn update(state: &mut f32, message: Message) {
    match message {
        Message::UpdateValue(val) => *state = val,
        Message::Tick => {}
    }
}

fn view(state: &'_ f32) -> Element<'_, Message> {
    container(
        column![
            text(format_f32(*state)),
            row![
                "Standard text input:",
                text_input::TextInput::new("placeholder", &state.to_string())
                    .on_input(|s| Message::UpdateValue(s.parse().unwrap_or_default()))
                    .width(100)
            ]
            .align_y(Center)
            .spacing(4.0),
            row![
                "Standard text input (fixed):",
                text_input::TextInput::new("placeholder", &format_f32(*state))
                    .on_input(|s| Message::UpdateValue(s.parse().unwrap_or_default()))
                    .width(100)
            ]
            .align_y(Center)
            .spacing(4.0),
            row![
                "Foray numeric input:",
                foray_widgets::numeric_input::NumericInput::new(*state)
                    .on_input(Message::UpdateValue)
                    .width(100)
            ]
            .align_y(Center)
            .spacing(4.0),
            button("tick").on_press(Message::Tick)
        ]
        .spacing(4.0)
        .width(Shrink)
        .align_x(Alignment::End),
    )
    .center(Fill)
    .into()
}

fn format_f32(val: f32) -> String {
    format!("{:.5}", val)
}
