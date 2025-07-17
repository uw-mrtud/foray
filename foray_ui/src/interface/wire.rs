use std::iter::once;

use crate::app::{Action, App};
use crate::gui_node::template_node_size;
use crate::math::Point;
use crate::style::theme::AppTheme;
use crate::StableMap;
use canvas::{Path, Stroke};
use foray_graph::graph::{PortRef, IO};
use iced::{widget::*, Size};

impl App {
    pub fn wire_curve(
        &'_ self,
        wire_end_node: u32,
        points: &StableMap<u32, Point>,
    ) -> Vec<(Path, Stroke<'_>)> {
        let port_position = |port: &PortRef| {
            let node_size = template_node_size(&self.network.graph.get_node(port.node).template);
            points[&port.node]
                + find_port_offset(port, self.network.graph.port_index(port), node_size).into()
        };

        //// Handle currently active wire
        // TODO: test nodes with multiple out ports
        let active_wire = match &self.action {
            Action::CreatingInputWire(input, Some(tentative_output)) => Some((
                (port_position(input), port_position(tentative_output)),
                active_wire_stroke(&self.app_theme, true),
            )),
            Action::CreatingInputWire(input, None) => Some((
                (
                    port_position(input),
                    self.cursor_position + self.network.shapes.camera.position,
                ),
                active_wire_stroke(&self.app_theme, false),
            )),
            Action::CreatingOutputWire(output, Some(input)) => Some((
                (port_position(input), port_position(output)),
                active_wire_stroke(&self.app_theme, true),
            )),
            Action::CreatingOutputWire(output, None) => Some((
                (
                    self.cursor_position + self.network.shapes.camera.position,
                    port_position(output),
                ),
                active_wire_stroke(&self.app_theme, false),
            )),
            _ => None,
        };

        //// Handle all wires
        let incoming_wires = self.network.graph.incoming_edges(&wire_end_node);
        incoming_wires
            .iter()
            .map(|(from, to)| {
                let stroke = wire_status(from, to, &self.action, &self.app_theme);
                ((port_position(to), port_position(from)), stroke)
            })
            //// include the active wire
            .chain(once(active_wire).flatten())
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
}

use super::node::{NODE_RADIUS, PORT_RADIUS};
use crate::app;
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

/// Determine the status of a given *non-active* wire, and provide the corresponding color
/// The current action determines how existing wires should be displayed, to provide
/// context about how the current action will affect other wires
pub fn wire_status<'a>(
    output: &PortRef,
    input: &PortRef,
    current_action: &app::Action,
    theme: &'a AppTheme,
) -> Stroke<'a> {
    assert!(output.io == IO::Out);
    assert!(input.io == IO::In);

    //let p = theme.extended_palette();

    let default_stroke = default_wire_stroke(theme);
    let maybe_delete = default_stroke.with_color(theme.danger.weak_color().into());
    let will_delete = with_dashed_stroke(maybe_delete);

    match current_action {
        app::Action::CreatingInputWire(active_input, active_output) => {
            //// if a new wire is created at an input, any existing wires will be deleted
            if active_input == input {
                //// differentiate between if the new wire is complete, and a MouseUp event
                //// would trigger wire deletion
                if active_output.is_some() {
                    will_delete
                } else {
                    maybe_delete
                }
            } else {
                default_stroke
            }
        }
        app::Action::CreatingOutputWire(_, None) => default_stroke,
        app::Action::CreatingOutputWire(_, Some(active_input)) => {
            //// if a new wire is created at an input, any existing wires will be deleted
            if active_input == input {
                will_delete
            } else {
                default_stroke
            }
        }
        app::Action::Idle => default_stroke,
        _ => default_stroke,
    }
}

/// active wire color
pub fn active_wire_stroke(t: &'_ AppTheme, is_tentative_connection: bool) -> Stroke<'_> {
    let stroke = default_wire_stroke(t).with_color(t.secondary.strong_color().into());
    if !is_tentative_connection {
        with_dashed_stroke(stroke)
    } else {
        stroke
    }
}

fn with_dashed_stroke(stroke: Stroke) -> Stroke {
    Stroke {
        line_dash: canvas::LineDash {
            segments: &[10.0],
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
