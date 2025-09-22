use std::marker::PhantomData;

use iced::{
    Element, Length, Rectangle, Renderer, Theme, Transformation, Vector, event,
    keyboard::Modifiers,
    mouse::{self, ScrollDelta},
    widget::canvas::Program,
};

use iced::widget::canvas;
use iced::widget::canvas::event::Event;

use crate::{camera::Camera, shape_context::ShapeContext};

impl<'a, M: 'a, P: Program<M, State = ShapeContext>> Into<Element<'a, M>> for NodeCanvas<'a, M, P> {
    fn into(self) -> Element<'a, M> {
        canvas(self).width(Length::Fill).height(Length::Fill).into()
    }
}

pub struct NodeCanvas<'a, M, P: Program<M, State = ShapeContext>> {
    nodes: &'a [((f32, f32), P)],
    camera: Camera,
    update_camera: Option<Box<dyn Fn(Camera) -> M + 'a>>,
    m: PhantomData<M>,
}

impl<'a, M, P: Program<M, State = ShapeContext>> NodeCanvas<'a, M, P> {
    pub fn new(nodes: &'a [((f32, f32), P)], camera: Camera) -> Self {
        NodeCanvas {
            nodes,
            camera,
            update_camera: None,
            m: PhantomData::default(),
        }
    }

    pub fn on_update_camera(mut self, update_camera: impl Fn(Camera) -> M + 'a) -> Self {
        self.update_camera = Some(Box::new(update_camera));
        self
    }
}

impl<Message, P: Program<Message, State = ShapeContext>> canvas::Program<Message>
    for NodeCanvas<'_, Message, P>
{
    type State = NodeCanvasState;

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        frame.scale(10.0);

        let geo = self
            .nodes
            .iter()
            .flat_map(|(position, node)| {
                node.draw(
                    &ShapeContext::new(self.camera, *position),
                    renderer,
                    theme,
                    bounds,
                    cursor,
                )
            })
            .collect();

        geo
    }

    fn update(
        &self,
        state: &mut Self::State,
        event: Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> (event::Status, Option<Message>) {
        match event {
            Event::Mouse(event) => match event {
                mouse::Event::WheelScrolled { delta } => {
                    if let Some(update_camera) = &self.update_camera {
                        let scroll_amount = match delta {
                            ScrollDelta::Lines { x, y } => (-x * 10.0, -y * 10.0),
                            ScrollDelta::Pixels { x, y } => (-x, -y),
                        };
                        let mut new_camera = self.camera;
                        if state.modifiers.control() {
                            //// Zoom
                            let zoom_scale = 500.0;
                            let zoom_min = 0.20;
                            let zoom_max = 8.00;
                            //// Zoom
                            let z_new = (new_camera.zoom * (1.0 + (-scroll_amount.1 / zoom_scale)))
                                .clamp(zoom_min, zoom_max);
                            new_camera.zoom = z_new;
                            //// We want to zoom in based on where the cursor is.
                            //// The cursor in world space should remain fixed, so we shift the
                            //// camera position to compensate.
                            if let Some(cursor_position) = cursor.position_in(bounds) {
                                let center_offset =
                                    Vector::new(bounds.center_x(), bounds.center_y());
                                let z_old = self.camera.zoom;
                                let scaled_cursor = (cursor_position - center_offset)
                                    * Transformation::scale(1.0 / z_old);

                                // This was derived by finding the world_cursor_position:
                                //
                                // world_cursor_position = ((cursor_position-center_offset)/z_old) + camera_position
                                //
                                // The amount to shift the camera position, should be somewhere in
                                // this direction:
                                //
                                // camera_shift = (world_cursor_position-camera_position) *
                                // correction_factor
                                //
                                // new_camera_position = camera_position - camera_shift
                                //
                                // new_world_cursor_position = ((cursor_position-center_offset)/z_new) + new_camera_position
                                //
                                // we can solve for correction_factor by setting
                                //
                                // new_world_cursor_position - world_cursor_position = 0
                                //
                                // (we want to the cursor to stay in the same world position as we zoom in)
                                let correction_factor =
                                    Transformation::scale((z_old / z_new) - 1.0);
                                let delta = scaled_cursor * correction_factor;
                                new_camera.pan((-delta.x, -delta.y));

                                // Should be zero!
                                // dbg!(
                                //     self.camera.cursor_to_world(cursor_position, bounds.size())
                                //         - new_camera
                                //             .cursor_to_world(cursor_position, bounds.size())
                                // );
                            };
                        } else {
                            //// Pan
                            new_camera.pan(scroll_amount)
                        }
                        return (event::Status::Captured, Some(update_camera(new_camera)));
                    }
                }
                _ => {}
            },
            Event::Touch(_event) => {}
            Event::Keyboard(event) => match event {
                iced::keyboard::Event::ModifiersChanged(modifiers) => state.modifiers = modifiers,
                _ => {}
            },
        }
        (event::Status::Ignored, None)
    }
}

#[derive(Default)]
pub struct NodeCanvasState {
    modifiers: Modifiers,
}
