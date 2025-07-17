use iced::{
    theme::{
        palette::{Background, Extended, Pair, Primary, Success},
        Palette,
    },
    Theme,
};
use serde::{Deserialize, Serialize};

use super::color::{mix, GuiColor};

#[derive(Clone, Serialize, Deserialize)]
pub struct AppTheme {
    pub background: GuiColor,
    pub text: GuiColor,
    pub primary: GuiColor,
    pub secondary: GuiColor,
    pub success: GuiColor,
    pub danger: GuiColor,
    // For Ports
    pub red: GuiColor,
    pub orange: GuiColor,
    pub green: GuiColor,
    pub cyan: GuiColor,
    pub blue: GuiColor,
}

impl Default for AppTheme {
    fn default() -> Self {
        AppTheme {
            background: GuiColor::new(16, 32, 32, 0.03, 0.2),
            text: GuiColor::new(240, 208, 192, 0.1, 0.4),
            primary: GuiColor::new(128, 144, 80, 0.3, 0.1),
            secondary: GuiColor::new(176, 144, 112, 0.1, 0.2),
            success: GuiColor::new(112, 128, 224, 0.1, 0.2),
            danger: GuiColor::new(176, 64, 64, 0.1, 0.2),
            red: GuiColor::new(175, 48, 41, 0.1, 0.2),
            orange: GuiColor::new(175, 125, 41, 0.1, 0.2),
            green: GuiColor::new(102, 128, 11, 0.1, 0.2),
            cyan: GuiColor::new(36, 131, 123, 0.1, 0.2),
            blue: GuiColor::new(32, 94, 166, 0.1, 0.2),
        }
    }
}
impl From<AppTheme> for Theme {
    fn from(app_theme: AppTheme) -> Self {
        let palette = Palette {
            background: app_theme.background.base_color.into(),
            text: app_theme.text.base_color.into(),
            primary: app_theme.primary.base_color.into(),
            success: app_theme.success.base_color.into(),
            danger: app_theme.danger.base_color.into(),
        };
        iced::Theme::custom_with_fn("flexoki".into(), palette, move |palette| {
            extended(palette, app_theme)
        })
    }
}

pub fn extended(_palette: Palette, app_theme: AppTheme) -> Extended {
    let AppTheme {
        background,
        text,
        primary,
        secondary,
        success,
        danger,
        ..
    } = app_theme;

    let base = |gui_color: GuiColor| Pair::new(gui_color.base_color.into(), text.base_color.into());

    let weak = |gui_color: GuiColor| {
        Pair::new(
            mix(
                gui_color.base_color,
                text.base_color,
                gui_color.weak_modifier,
            )
            .into(),
            text.base_color.into(),
        )
    };
    let strong = |gui_color: GuiColor| {
        Pair::new(
            mix(
                gui_color.base_color,
                text.base_color,
                gui_color.strong_modifier,
            )
            .into(),
            text.base_color.into(),
        )
    };

    Extended {
        background: Background {
            base: base(background),
            weak: weak(background),
            strong: strong(background),
        },
        primary: Primary {
            base: base(primary),
            weak: weak(primary),
            strong: strong(primary),
        },
        secondary: iced::theme::palette::Secondary {
            base: base(secondary),
            weak: weak(secondary),
            strong: strong(secondary),
        },
        success: Success {
            base: base(success),
            weak: weak(success),
            strong: strong(success),
        },
        danger: iced::theme::palette::Danger {
            base: base(danger),
            weak: weak(danger),
            strong: strong(danger),
        },
        is_dark: true,
    }
}
