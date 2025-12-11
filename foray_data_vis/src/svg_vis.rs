use serde::{Deserialize, Serialize};

use iced::advanced::svg::Svg;
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SvgVis {
    #[serde(skip, default = "crate::series_vis::default_svg")]
    pub(crate) svg: Svg,
}

impl SvgVis {
    pub fn new(svg_string: &str) -> Self {
        Self {
            svg: iced::advanced::svg::Svg::new(iced::advanced::svg::Handle::from_memory(
                svg_string.to_string().into_bytes(),
            )),
        }
    }

    pub fn svg(&self) -> &Svg {
        &self.svg
    }
}
