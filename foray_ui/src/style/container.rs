use iced::{border, widget::container::Style, Theme};

/// A rounded [`Container`] with a background.
pub fn rounded_box(theme: &Theme) -> Style {
    let palette = theme.extended_palette();

    Style {
        background: Some(palette.background.weak.color.into()),
        border: border::rounded(4),
        ..Style::default()
    }
}
