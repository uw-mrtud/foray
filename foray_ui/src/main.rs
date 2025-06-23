use iced::Element;
use iced::widget::{button, text};

pub fn main() -> iced::Result {
    iced::run("A cool counter", update, view)
}
fn update(counter: &mut u64, message: Message) {
    match message {
        Message::Increment => *counter += 1,
    }
}

fn view(counter: &'_ u64) -> Element<'_, Message> {
    button(text(counter)).on_press(Message::Increment).into()
}
#[derive(Debug, Clone)]
enum Message {
    Increment,
}
