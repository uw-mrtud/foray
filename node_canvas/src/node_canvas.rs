use std::marker::PhantomData;

use iced::{
    Element, Length, Rectangle, Renderer, Size, Theme, Transformation, Vector, event,
    keyboard::Modifiers,
    mouse::{self, ScrollDelta},
    widget::{
        canvas::{Frame, Program},
        center,
    },
};

use iced::widget::canvas;
use iced::widget::canvas::event::Event;
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
        let center_offset = Vector::ZERO; //Vector::new(bounds.center_x(), bounds.center_y());

        let camera_translation = Vector::new(self.camera.position.0, self.camera.position.1); // + center_offset;
        let camera_scale = self.camera.zoom;

        let shape_translation = Vector::new(self.position.0, self.position.1);

        frame.translate(center_offset);
        frame.scale(camera_scale);

        frame.translate(-camera_translation);
        frame.translate(shape_translation);

        // frame.translate(-camera_translation);
        // frame.translate(shape_translation * camera_scale);
        // frame.scale(camera_scale);
        // frame.translate(-center_offset);
        // frame.translate(center_offset * transform.scale_factor());
        // frame.scale(camera_scale);
        // frame.translate(-center_offset * (1.0 / camera_scale));
        frame
    }
}

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
            m: Default::default(),
            camera,
            update_camera: None,
        }
    }

    pub fn on_update_camera(mut self, update_camera: impl Fn(Camera) -> M + 'a) -> Self {
        self.update_camera = Some(Box::new(update_camera));
        self
    }
}

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
    // pub fn transform(&self) -> Transformation {
    //     Transformation::translate(self.position.0, self.position.1)
    //         * Transformation::scale(self.zoom)
    // }
    //
    // pub fn zoom_in(&mut self) {
    //     self.zoom *= 1.2;
    // }
    // pub fn zoom_out(&mut self) {
    //     self.zoom *= 0.8;
    // }
    pub fn cursor_to_world(&self, point: iced::Point, canvas_size: Size) -> iced::Point {
        let center_offset = Vector::ZERO; //Vector::new(canvas_size.width, canvas_size.height) * 0.5;

        let camera_translation = Vector::new(self.position.0, self.position.1); // + center_offset;

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

impl<Message, P: Program<Message, State = ShapeContext>> canvas::Program<Message>
    for NodeCanvas<'_, Message, P>
{
    type State = NodeCanvasState;

    fn draw(
        &self,
        state: &Self::State,
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
                            new_camera.zoom *= 1.0 + (-scroll_amount.1 / 100.0);
                            if let Some(canvas_cursor) = cursor.position_in(bounds) {
                                let world_cursor =
                                    canvas_cursor * Transformation::scale(1.0 / self.camera.zoom); // self.camera.cursor_to_world(canvas_cursor, bounds.size());

                                let delta = world_cursor
                                    // - iced::Point::new(
                                    //     self.camera.position.0,
                                    //     self.camera.position.1,
                                    // ))
                                    * Transformation::scale(1.0 - (new_camera.zoom / self.camera.zoom));
                                new_camera.pan((-delta.x, -delta.y));

                                let new_world_curosr =
                                    new_camera.cursor_to_world(canvas_cursor, bounds.size());

                                dbg!(world_cursor - new_world_curosr);
                            };
                        } else {
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
