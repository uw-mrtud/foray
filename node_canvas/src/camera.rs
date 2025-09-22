use iced::{Point, Size, Transformation, Vector};

#[derive(Debug, Clone, Copy)]
pub struct Camera {
    pub position: (f32, f32),
    pub zoom: f32,
}
impl Camera {
    pub fn pan(&mut self, movement: (f32, f32)) {
        self.position.0 += movement.0;
        self.position.1 += movement.1;
    }
    pub fn cursor_to_world(&self, point: Point, canvas_size: Size) -> Point {
        let center_offset = Vector::new(canvas_size.width, canvas_size.height) * 0.5;

        let camera_translation = Vector::new(self.position.0, self.position.1);

        (point - center_offset) * Transformation::scale(1.0 / self.zoom) + camera_translation
    }
}
impl Default for Camera {
    fn default() -> Self {
        Self {
            position: Default::default(),
            zoom: 1.0,
        }
    }
}
