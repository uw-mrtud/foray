use iced::Size;
use serde::{Deserialize, Serialize};

use crate::math::{Point, Vector};

#[derive(Clone, Serialize, Deserialize, Debug, Copy)]
pub struct Camera {
    pub position: Vector,
    pub zoom: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: [0., 0.].into(),
            zoom: 1.0,
        }
    }
}
impl Camera {
    pub fn pan(&mut self, movement: (f32, f32)) {
        self.position.x += movement.0;
        self.position.y += movement.1;
    }
    pub fn cursor_to_world(&self, point: Point, canvas_size: Size) -> Point {
        let center_offset = Vector::new(canvas_size.width, canvas_size.height) * 0.5;

        let camera_translation = Vector::new(self.position.x, self.position.y);

        (point - center_offset) * (1.0 / self.zoom) + camera_translation
    }
}
