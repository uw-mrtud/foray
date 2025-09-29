use iced::{
    event,
    keyboard::Modifiers,
    mouse::{self, Cursor, ScrollDelta},
    Element, Length, Rectangle, Renderer, Theme, Transformation,
};

use iced::widget::canvas;
use iced::widget::canvas::event::Event;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::{
    interface::node::draw_node,
    math::Point,
    style::theme::AppTheme,
    workspace::{Workspace, WorkspaceMessage},
};

use super::camera::Camera;

#[derive(Serialize, Deserialize, Clone)]
pub struct State {
    pub camera: Camera,
    pub shape_positions: IndexMap<u32, Point>,
}

impl Default for State {
    fn default() -> Self {
        Self::new([].into())
    }
}

impl State {
    pub fn new(shapes: IndexMap<u32, Point>) -> State {
        Self {
            camera: Camera::default(),
            shape_positions: shapes,
        }
    }
}

pub fn node_canvas<'a>(
    shapes: &'a IndexMap<u32, Point>,
    camera: Camera,
    workspace: &'a Workspace,
    app_theme: &'a AppTheme,
) -> Element<'a, WorkspaceMessage> {
    canvas(NodeCanvas::new(shapes, camera, workspace, app_theme))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// Short lived PRIVATE struct used for drawing the canvas/nodes.
/// Needs to be a struct because we have to implement canvas::Program
/// This is why we are just storing references here
/// Don't expose this struct, as accidentally keeping these references around
/// could get very messy
struct NodeCanvas<'a> {
    positions: &'a IndexMap<u32, Point>,
    camera: Camera,
    workspace: &'a Workspace,
    app_theme: &'a AppTheme,
}

impl<'a> NodeCanvas<'a> {
    pub fn new(
        positions: &'a IndexMap<u32, Point>,
        camera: Camera,
        workspace: &'a Workspace,
        app_theme: &'a AppTheme,
    ) -> NodeCanvas<'a> {
        NodeCanvas {
            positions,
            camera,
            workspace,
            app_theme,
        }
    }
}

impl<'a> canvas::Program<WorkspaceMessage> for NodeCanvas<'a> {
    type State = NodeCanvasState;

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        frame.translate(iced::Vector::from(self.camera.center_offset()) * -1.0);
        frame.scale(self.camera.zoom);
        frame.translate((-self.camera.position).into());

        let world_cursor_position = match cursor.position() {
            Some(p) => Cursor::Available((self.camera.cursor_to_world(p.into())).into()),
            None => Cursor::Unavailable,
        };
        //// Wires
        let wire_geometry = self.positions.iter().flat_map(|(id, _p)| {
            self.workspace
                .create_wires(*id, self.positions, world_cursor_position, self.app_theme)
        });
        wire_geometry.for_each(|(p, s)| frame.stroke(&p, s.with_width(2.0 * self.camera.zoom)));

        //// Nodes
        self.positions.iter().for_each(|(id, position)| {
            frame.with_save(|frame| {
                frame.translate((*position).into());

                let node_cursor_position = match cursor.position() {
                    Some(p) => Cursor::Available(
                        (self.camera.cursor_to_world(p.into()) - position.to_vector()).into(),
                    ),
                    None => Cursor::Unavailable,
                };
                let node = self.workspace.network.graph.get_node(*id);
                let is_selected = self.workspace.network.selected_shapes.contains(id);

                draw_node(
                    frame,
                    node_cursor_position,
                    self.camera.zoom,
                    node,
                    *id,
                    self.workspace.action.clone(),
                    is_selected,
                    self.app_theme,
                );
            });
        });

        vec![frame.into_geometry()]
    }

    fn update(
        &self,
        state: &mut Self::State,
        event: Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> (event::Status, Option<WorkspaceMessage>) {
        let world_cursor_position = match cursor.position() {
            Some(p) => self.camera.cursor_to_world(p.into()),
            None => self.workspace.cursor_position, //return (event::Status::Ignored, None),
        };

        match event {
            Event::Mouse(event) => match event {
                mouse::Event::WheelScrolled { delta } => {
                    let scroll_amount = match delta {
                        ScrollDelta::Lines { x, y } => (-x * 4.0, -y * 4.0),
                        ScrollDelta::Pixels { x, y } => (-x, -y),
                    };
                    let mut new_camera = self.camera;
                    if state.modifiers.control() {
                        let zoom_scale = 400.0;
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
                            let z_old = self.camera.zoom;
                            let scaled_cursor = (cursor_position
                                + self.camera.center_offset().into())
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
                            let correction_factor = Transformation::scale((z_old / z_new) - 1.0);
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
                    return (
                        event::Status::Captured,
                        Some(WorkspaceMessage::UpdateCamera(new_camera)),
                    );
                }
                mouse::Event::ButtonPressed(mouse::Button::Left) => {
                    for (node_id, position) in self.positions.iter() {
                        let node = self.workspace.network.graph.get_node(*node_id);
                        let node_cursor_position: iced::Point =
                            (world_cursor_position - position.to_vector()).into();

                        // Node Collision
                        if node.node_bounding_rect().contains(node_cursor_position) {
                            return (
                                event::Status::Captured,
                                Some(WorkspaceMessage::OnCanvasDown(Some(*node_id))),
                            );
                        }
                        // Port Collision
                        let (input_ports, output_ports) = node.port_positions(*node_id);
                        for (port_rect, port_ref, _) in input_ports.into_iter().chain(output_ports)
                        {
                            if port_rect.contains(node_cursor_position) {
                                return (
                                    event::Status::Captured,
                                    Some(WorkspaceMessage::PortPress(port_ref)),
                                );
                            }
                        }
                    }
                    return (
                        event::Status::Captured,
                        Some(WorkspaceMessage::OnCanvasDown(None)),
                    );
                }
                mouse::Event::ButtonPressed(mouse::Button::Right) => {
                    for (node_id, position) in self.positions.iter() {
                        let node = self.workspace.network.graph.get_node(*node_id);
                        let node_cursor_position: iced::Point =
                            (world_cursor_position - position.to_vector()).into();

                        // Port Collision
                        let (input_ports, output_ports) = node.port_positions(*node_id);
                        for (port_rect, port_ref, _) in input_ports.into_iter().chain(output_ports)
                        {
                            if port_rect.contains(node_cursor_position) {
                                return (
                                    event::Status::Captured,
                                    Some(WorkspaceMessage::PortDelete(port_ref)),
                                );
                            }
                        }
                    }
                }
                mouse::Event::ButtonReleased(mouse::Button::Left) => {
                    for (node_id, position) in self.positions.iter() {
                        let node = self.workspace.network.graph.get_node(*node_id);
                        let node_cursor_position: iced::Point =
                            (world_cursor_position - position.to_vector()).into();

                        // Port Collision
                        let (input_ports, output_ports) = node.port_positions(*node_id);
                        for (port_rect, port_ref, _) in input_ports.into_iter().chain(output_ports)
                        {
                            if port_rect.contains(node_cursor_position) {
                                return (
                                    event::Status::Captured,
                                    Some(WorkspaceMessage::PortMouseUp(port_ref)),
                                );
                            }
                        }
                    }
                    return (event::Status::Captured, Some(WorkspaceMessage::OnCanvasUp));
                }
                mouse::Event::CursorMoved { position } => {
                    return (
                        event::Status::Captured,
                        Some(WorkspaceMessage::OnMove(position.into())),
                    )
                }
                _ => {}
            },
            Event::Touch(_event) => {}
            Event::Keyboard(event) => match event {
                iced::keyboard::Event::ModifiersChanged(modifiers) => state.modifiers = modifiers,
                _ => {}
            },
        }

        if self.camera.bounds_size != bounds.size() {
            return (
                event::Status::Captured,
                Some(WorkspaceMessage::UpdateCamera(Camera {
                    bounds_size: bounds.size(),
                    ..self.camera
                })),
            );
        }

        (event::Status::Ignored, None)
    }
}

#[derive(Default)]
pub struct NodeCanvasState {
    modifiers: Modifiers,
}
