use palette::{rgb::Rgb, Mix};
use serde::{Deserialize, Serialize};
const WHITE: Color = Color {
    r: 15. / 16.,
    g: 13. / 16.,
    b: 12. / 16.,
    a: 1.0,
};
//const BLACK: Color = Color {
//    r: 1.0,
//    g: 1.0,
//    b: 1.0,
//    a: 1.0,
//};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}
impl Color {
    pub fn iced_color(&self) -> iced::Color {
        Into::<iced::Color>::into(*self)
    }
    pub fn into_rbg8(&self) -> (u8, u8, u8) {
        let to_u8 = |v| (v * 255.0) as u8;
        (to_u8(self.r), to_u8(self.g), to_u8(self.b))
    }
}

impl From<iced::Color> for Color {
    fn from(v: iced::Color) -> Self {
        let iced::Color { r, g, b, a } = v;
        Self { r, g, b, a }
    }
}
impl From<Color> for iced::Color {
    fn from(v: Color) -> Self {
        let Color { r, g, b, a } = v;
        Self { r, g, b, a }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct GuiColor {
    pub base_color: Color,
    pub weak_modifier: f32,
    pub strong_modifier: f32,
}

impl GuiColor {
    pub fn new(r: u8, g: u8, b: u8, weak_modifier: f32, strong_modifier: f32) -> Self {
        let color = iced::Color::from_rgba8(r, g, b, 1.0);
        Self {
            base_color: color.into(),
            weak_modifier,
            strong_modifier,
        }
    }

    pub fn weak_color(&self) -> Color {
        mix(self.base_color, WHITE, self.weak_modifier)
    }
    pub fn strong_color(&self) -> Color {
        mix(self.base_color, WHITE, self.strong_modifier)
    }
    pub fn color_pair(&self) -> (Color, Color) {
        (self.base_color, self.strong_color())
    }
}

pub fn mix(a: Color, b: Color, factor: f32) -> Color {
    let a: iced::Color = a.into();
    let b: iced::Color = b.into();
    let a_lin = Rgb::from(a).into_linear();
    let b_lin = Rgb::from(b).into_linear();

    let mixed = a_lin.mix(b_lin, factor);
    let ic: iced::Color = Rgb::from_linear(mixed).into();
    ic.into()
}
