use iced::alignment::Horizontal::Right;
use iced::widget::container::background;
use iced::widget::scrollable::{Direction, Scrollbar};
use iced::widget::{container, rule, space};
use iced::{
    widget::{column, row, scrollable, slider, text},
    Element,
};

use crate::app::Message;
use crate::style::color::{Color, GuiColor};
use crate::style::theme::AppTheme;

use super::SEPERATOR;

#[derive(Clone, Debug)]
pub enum GuiColorMessage {
    R(f32),
    G(f32),
    B(f32),
    Weak(f32),
    Strong(f32),
}

//TODO: it might be nicer to wrap GuiColorMessage inside the variants of this enum.
// It just requries a bit of manipulation to get pass info down to GuiControl `view`
// so that it can construct it's Message types.
// I think wrapping would give us more flexibility for different data types for each event.
// (not thought through super well yet)
#[derive(Clone, Debug)]
pub enum AppThemeMessage {
    Primary,
    Secondary,
    Sucess,
    Danger,
    Background,
    Text,
}

impl AppTheme {
    pub fn update(&mut self, theme_message: AppThemeMessage, theme_value: GuiColorMessage) {
        match theme_message {
            AppThemeMessage::Primary => self.primary.update(theme_value),
            AppThemeMessage::Secondary => self.secondary.update(theme_value),
            AppThemeMessage::Sucess => self.success.update(theme_value),
            AppThemeMessage::Danger => self.danger.update(theme_value),
            AppThemeMessage::Background => self.background.update(theme_value),
            AppThemeMessage::Text => self.text.update(theme_value),
        }
    }

    /// Debug view for editing themes, not intended to be end user facing
    pub fn view(&'_ self) -> Element<'_, Message> {
        let color_element = move |color: Color| {
            container(text(""))
                .style(move |_t| background(color.iced_color()))
                .width(50.)
                .height(20.)
        };
        let color_control = |gui_color: GuiColor, lbl: String, theme_message: AppThemeMessage| {
            column![
                row![
                    text(lbl),
                    space::horizontal(),
                    color_element(gui_color.base_color),
                    color_element(gui_color.weak_color()),
                    color_element(gui_color.strong_color()),
                ]
                .spacing(10.),
                gui_color.view(theme_message)
            ]
            .padding(15.)
            .width(350.)
        };

        scrollable(
            row![
                color_control(self.primary, "Primary".into(), AppThemeMessage::Primary),
                rule::vertical(SEPERATOR),
                color_control(
                    self.secondary,
                    "Secondary".into(),
                    AppThemeMessage::Secondary
                ),
                rule::vertical(SEPERATOR),
                color_control(self.success, "Success".into(), AppThemeMessage::Sucess),
                rule::vertical(SEPERATOR),
                color_control(self.danger, "Danger".into(), AppThemeMessage::Danger),
                rule::vertical(SEPERATOR),
                color_control(
                    self.background,
                    "Background".into(),
                    AppThemeMessage::Background
                ),
                rule::vertical(SEPERATOR),
                color_control(self.text, "Text".into(), AppThemeMessage::Text),
            ]
            .height(130),
        )
        .direction(Direction::Horizontal(Scrollbar::default()))
        .into()
    }
}

impl GuiColor {
    pub fn update(&mut self, message: GuiColorMessage) {
        match message {
            GuiColorMessage::R(r) => self.base_color.r = r / 16.0,
            GuiColorMessage::G(g) => self.base_color.g = g / 16.0,
            GuiColorMessage::B(b) => self.base_color.b = b / 16.0,
            GuiColorMessage::Weak(w) => self.weak_modifier = w,
            GuiColorMessage::Strong(s) => self.strong_modifier = s,
        }
    }
    pub fn view<'a>(self, theme_message: AppThemeMessage) -> Element<'a, Message> {
        let GuiColor {
            base_color,
            weak_modifier,
            strong_modifier,
        } = self;

        fn color_control<'a>(
            lbl: &str,
            v: f32,
            // This is a bit verbose, but is just a nice way to produce GuiColorMessages
            update: impl Fn(f32) -> GuiColorMessage + 'a,
            theme_message: AppThemeMessage,
        ) -> iced::widget::Row<'a, Message> {
            row![
                text(format!("{lbl}: {}", (v).floor() as i32))
                    .width(60.)
                    .align_x(Right),
                slider(0.0..=16.0, v, move |v| Message::ThemeValueChange(
                    theme_message.clone(),
                    update(v)
                ))
                .step(1.)
            ]
            .spacing(5.0)
        }
        fn mod_control<'a>(
            lbl: &str,
            v: f32,
            update: impl Fn(f32) -> GuiColorMessage + 'a,
            theme_message: AppThemeMessage,
        ) -> iced::widget::Row<'a, Message> {
            row![
                text(format!("{lbl}: {v:.2}")).width(60.).align_x(Right),
                slider(0.0..=1.0, v, move |v| Message::ThemeValueChange(
                    theme_message.clone(),
                    update(v)
                ))
                .step(0.01)
            ]
            .spacing(5.0)
        }

        row![
            column![
                color_control(
                    "r",
                    base_color.r * 16.,
                    GuiColorMessage::R,
                    theme_message.clone()
                ),
                color_control(
                    "g",
                    base_color.g * 16.,
                    GuiColorMessage::G,
                    theme_message.clone()
                ),
                color_control(
                    "b",
                    base_color.b * 16.,
                    GuiColorMessage::B,
                    theme_message.clone()
                ),
            ],
            column![
                mod_control(
                    "w",
                    weak_modifier,
                    GuiColorMessage::Weak,
                    theme_message.clone()
                ),
                mod_control(
                    "s",
                    strong_modifier,
                    GuiColorMessage::Strong,
                    theme_message.clone()
                )
            ]
        ]
        .spacing(10.)
        .into()
    }
}
