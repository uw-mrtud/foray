use std::iter::once;

use crate::style::theme::AppTheme;
use crate::workspace::Workspace;
use crate::{math::Point, workspace::Action};
use canvas::{Path, Stroke};
use foray_graph::graph::{PortRef, IO};
use iced::mouse::Cursor;
use iced::{widget::*, Size};
use indexmap::IndexMap;

impl Workspace {
    pub fn create_wires<'a>(
        &'a self,
        wire_end_node: u32,
        points: &IndexMap<u32, Point>,
        world_cursor_position: Cursor,
        app_theme: &'a AppTheme,
    ) -> Vec<(Path, Stroke<'a>)> {
        let port_position = |port: &PortRef| {
            let node = self.network.graph.get_node(port.node);
            let index = self.network.graph.port_index(port);

            let port_center = match port.io {
                IO::In => node.input_port_bounding(index).center(),
                IO::Out => node.output_port_bounding(index).center(),
            };
            (port_center + points[&port.node].to_vector().into()).into()
        };

        //// Handle currently active wire
        // TODO: test nodes with multiple out ports
        let wire_creation_state = match (&self.action, world_cursor_position.position()) {
            (Action::CreatingInputWire(input, _), Some(world_cursor)) => {
                let hovered_output_port = self.network.graph.nodes_ref().iter().find_map(|n_id| {
                    let node = self.network.graph.get_node(*n_id);
                    let node_cursor_position = Point::from(world_cursor) - points[n_id].to_vector();

                    node.port_positions(*n_id)
                        .1
                        .into_iter()
                        .find_map(|(rect, port_ref, _)| {
                            match rect.contains(node_cursor_position.into()) {
                                true => Some(port_ref),
                                false => None,
                            }
                        })
                });
                WireCreationState::CreatingInput(input.clone(), hovered_output_port)
            }
            (Action::CreatingOutputWire(output, _), Some(world_cursor)) => {
                let hovered_input_port = self.network.graph.nodes_ref().iter().find_map(|n_id| {
                    let node = self.network.graph.get_node(*n_id);
                    let node_cursor_position = Point::from(world_cursor) - points[n_id].to_vector();

                    node.port_positions(*n_id)
                        .0
                        .into_iter()
                        .find_map(|(rect, port_ref, _)| {
                            match rect.contains(node_cursor_position.into()) {
                                true => Some(port_ref),
                                false => None,
                            }
                        })
                });
                WireCreationState::CreatingOutput(output.clone(), hovered_input_port)
            }
            _ => WireCreationState::Idle,
        };

        //// Handle all wires
        let incoming_wires = self.network.graph.incoming_edges(&wire_end_node);
        incoming_wires
            .iter()
            .map(|(output, input)| {
                let stroke = wire_status(output, input, &wire_creation_state, app_theme);
                ((port_position(input), port_position(output)), stroke)
            })
            //// include the active wire
            .chain(
                once(match &wire_creation_state {
                    WireCreationState::CreatingInput(input, Some(output)) => Some((
                        (port_position(&input), port_position(&output)),
                        wire_status(&output, &input, &wire_creation_state, app_theme),
                    )),
                    WireCreationState::CreatingOutput(output, Some(input)) => Some((
                        (port_position(&input), port_position(&output)),
                        wire_status(&output, &input, &wire_creation_state, app_theme),
                    )),
                    WireCreationState::CreatingInput(input, None) => Some((
                        (
                            port_position(&input),
                            world_cursor_position.position().unwrap(),
                        ),
                        active_wire_stroke(app_theme, false),
                    )),
                    WireCreationState::CreatingOutput(output, None) => Some((
                        (
                            world_cursor_position.position().unwrap(),
                            port_position(&output),
                        ),
                        active_wire_stroke(app_theme, false),
                    )),
                    WireCreationState::Idle => None,
                })
                .flatten(),
            )
            //// build the wire curves
            .map(|((from, to), stroke)| {
                (
                    Path::new(|builder| {
                        builder.move_to(from.into());
                        let mid = f32::abs((to.y - from.y) * 0.5).max(PORT_RADIUS * 2.);
                        builder.bezier_curve_to(
                            (from.x, from.y - mid).into(),
                            (to.x, to.y + mid).into(),
                            to.into(),
                        );
                    }),
                    stroke,
                )
            })
            .collect()
    }

    // pub fn wire_curve<'a>(
    //     &'a self,
    //     wire_end_node: u32,
    //     points: &IndexMap<u32, Point>,
    //     world_cursor_position: Cursor,
    //     app_theme: &'a AppTheme,
    // ) -> Vec<(Path, Stroke<'a>)> {
    //     let port_position = |port: &PortRef| {
    //         let node = self.network.graph.get_node(port.node);
    //         let index = self.network.graph.port_index(port);
    //
    //         let port_center = match port.io {
    //             IO::In => node.input_port_bounding(index).center(),
    //             IO::Out => node.output_port_bounding(index).center(),
    //         };
    //         (port_center + points[&port.node].to_vector().into()).into()
    //     };
    //
    //     //// Handle currently active wire
    //     // TODO: test nodes with multiple out ports
    //     let active_wire = match &self.action {
    //         Action::CreatingInputWire(input, Some(tentative_output)) => Some((
    //             (port_position(input), port_position(tentative_output)),
    //             active_wire_stroke(app_theme, true),
    //         )),
    //         Action::CreatingInputWire(input, None) => Some((
    //             (
    //                 port_position(input),
    //                 world_cursor_position.position().unwrap_or_default(), // self.cursor_position + self.network.shapes.camera.position,
    //             ),
    //             active_wire_stroke(app_theme, false),
    //         )),
    //         Action::CreatingOutputWire(output, Some(input)) => Some((
    //             (port_position(input), port_position(output)),
    //             active_wire_stroke(app_theme, true),
    //         )),
    //         Action::CreatingOutputWire(output, None) => Some((
    //             (
    //                 // self.cursor_position + self.network.shapes.camera.position,
    //                 world_cursor_position.position().unwrap_or_default(),
    //                 port_position(output),
    //             ),
    //             active_wire_stroke(app_theme, false),
    //         )),
    //         _ => None,
    //     };
    //
    //     //// Handle all wires
    //     let incoming_wires = self.network.graph.incoming_edges(&wire_end_node);
    //     incoming_wires
    //         .iter()
    //         .map(|(from, to)| {
    //             let stroke = wire_status(from, to, &self.action, app_theme);
    //             ((port_position(to), port_position(from)), stroke)
    //         })
    //         //// include the active wire
    //         .chain(once(active_wire).flatten())
    //         //// build the wire curves
    //         .map(|((from, to), stroke)| {
    //             (
    //                 Path::new(|builder| {
    //                     builder.move_to(from.into());
    //                     let mid = f32::abs((to.y - from.y) * 0.5).max(PORT_RADIUS * 2.);
    //                     builder.bezier_curve_to(
    //                         (from.x, from.y - mid).into(),
    //                         (to.x, to.y + mid).into(),
    //                         to.into(),
    //                     );
    //                 }),
    //                 stroke,
    //             )
    //         })
    //         .collect()
    // }
}

use super::node::{NODE_RADIUS, PORT_RADIUS};
use iced::Vector;

/// Determine where a port should be positioned relative to the origin of the node
pub fn find_port_offset(port_ref: &PortRef, port_index: usize, size: Size) -> Vector {
    let port_x = |i: usize| i as f32 * (size.width / 4.) + NODE_RADIUS * 2.;
    match port_ref.io {
        IO::In => Vector::new(port_x(port_index), 0.) + Vector::new(PORT_RADIUS, -PORT_RADIUS / 2.),
        IO::Out => {
            Vector::new(port_x(port_index), size.height)
                + Vector::new(PORT_RADIUS, PORT_RADIUS / 2.)
        }
    }
}

enum WireCreationState {
    CreatingInput(PortRef, Option<PortRef>),
    CreatingOutput(PortRef, Option<PortRef>),
    Idle,
}

/// Determine the status of a given *non-active* wire, and provide the corresponding color
/// The current action determines how existing wires should be displayed, to provide
/// context about how the current action will affect other wires
fn wire_status<'a>(
    output: &PortRef,
    input: &PortRef,
    current_action: &WireCreationState,
    theme: &'a AppTheme,
) -> Stroke<'a> {
    assert!(output.io == IO::Out);
    assert!(input.io == IO::In);

    //let p = theme.extended_palette();

    let default_stroke = default_wire_stroke(theme);
    let maybe_delete = default_stroke.with_color(theme.danger.weak_color().into());
    let will_delete = with_dashed_stroke(maybe_delete);

    let will_create = active_wire_stroke(theme, true);

    match current_action {
        WireCreationState::CreatingInput(active_input, None) => {
            //// if a new wire is created at an input, any existing wires will be deleted
            if active_input == input {
                maybe_delete
            } else {
                default_stroke
            }
        }
        WireCreationState::CreatingInput(active_input, Some(active_output)) => {
            if active_input == input && active_output == output {
                return will_create;
            }

            //// if a new wire is created at an input, any existing wires will be deleted
            if active_input == input {
                //// differentiate between if the new wire is complete, and a MouseUp event
                //// would trigger wire deletion
                will_delete
            } else {
                default_stroke
            }
        }
        WireCreationState::CreatingOutput(_, None) => default_stroke,
        WireCreationState::CreatingOutput(active_output, Some(active_input)) => {
            if active_input == input && active_output == output {
                return will_create;
            }

            //// if a new wire is created at an input, any existing wires will be deleted
            if active_input == input && active_output != output {
                will_delete
            } else {
                default_stroke
            }
        }
        WireCreationState::Idle => default_stroke,
    }
}

/// active wire color
pub fn active_wire_stroke(t: &'_ AppTheme, is_tentative_connection: bool) -> Stroke<'_> {
    let stroke = default_wire_stroke(t).with_color(t.green.weak_color().into());
    if !is_tentative_connection {
        with_dashed_stroke(stroke)
    } else {
        stroke
    }
}

fn with_dashed_stroke(stroke: Stroke) -> Stroke {
    Stroke {
        line_dash: canvas::LineDash {
            segments: &[5.0],
            offset: 0,
        },
        ..stroke
    }
}

pub fn default_wire_stroke(theme: &'_ AppTheme) -> Stroke<'_> {
    Stroke::default()
        .with_width(3.0)
        .with_color(theme.secondary.base_color.into())
        .with_line_cap(canvas::LineCap::Round)
}
