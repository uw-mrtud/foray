use iced::Size;
use serde::{Deserialize, Serialize};

use crate::math::{Point, Vector};

#[derive(Clone, Serialize, Deserialize, Debug, Copy)]
pub struct Camera {
    pub position: Vector,
    pub zoom: f32,
    #[serde(skip)]
    pub bounds_size: Size,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: [0., 0.].into(),
            zoom: 1.0,
            bounds_size: Size::new(100.0, 100.0),
        }
    }
}
impl Camera {
    pub fn pan(&mut self, movement: (f32, f32)) {
        self.position.x += movement.0;
        self.position.y += movement.1;
    }
    pub fn cursor_to_world(&self, point: Point) -> Point {
        let camera_translation = Vector::new(self.position.x, self.position.y);

        (point + self.center_offset()) * (1.0 / self.zoom) + camera_translation
    }
    pub fn center_offset(&self) -> Vector {
        Vector::new(-self.bounds_size.width, -self.bounds_size.height) * 0.5
    }
}
