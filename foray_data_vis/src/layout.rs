use iced::{
    Element,
    Length::Shrink,
    widget::{column, container, row, rule},
};

pub fn section<'a, M: 'a>(label: &'a str, content: Element<'a, M>) -> Element<'a, M> {
    column![
        label,
        container(rule::horizontal(1.0)).width(10.0),
        row![rule::vertical(1.0), container(content).padding(4.0)],
        container(rule::horizontal(1.0)).width(10.0),
    ]
    .height(Shrink)
    .into()
}
