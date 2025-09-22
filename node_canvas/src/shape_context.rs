use iced::{Renderer, Vector, widget::canvas::Frame};

use iced::widget::canvas;

use crate::camera::Camera;

#[derive(Default)]
pub struct ShapeContext {
    pub camera: Camera,
    pub position: (f32, f32),
}

impl ShapeContext {
    pub fn new(camera: Camera, position: (f32, f32)) -> Self {
        Self { camera, position }
    }

    pub fn frame_in_shape_space(
        &self,
        renderer: &Renderer,
        bounds: iced::Rectangle,
    ) -> Frame<Renderer> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let center_offset = Vector::new(bounds.center_x(), bounds.center_y());

        let camera_translation = Vector::new(self.camera.position.0, self.camera.position.1);
        let camera_scale = self.camera.zoom;

        let shape_translation = Vector::new(self.position.0, self.position.1);

        frame.translate(center_offset);
        frame.scale(camera_scale);

        frame.translate(-camera_translation);
        frame.translate(shape_translation);

        frame
    }
}
