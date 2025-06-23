use foray_ui::app::{subscriptions, theme, title, App};
use iced::{application, Font};

pub fn main() -> iced::Result {
    env_logger::init();

    application(title, App::update, App::view)
        .subscription(subscriptions)
        .theme(theme)
        .window(iced::window::Settings {
            min_size: Some((400., 300.).into()),
            ..Default::default()
        })
        .antialiasing(true)
        .window_size((1000., 800.))
        .decorations(true)
        .scale_factor(|_| 1.25)
        .font(include_bytes!("../data/CaskaydiaCoveNerdFont.ttf").as_slice())
        .font(include_bytes!("../data/CaskaydiaCove.ttf").as_slice())
        .default_font(Font::with_name("CaskaydiaCove"))
        .run()
}
