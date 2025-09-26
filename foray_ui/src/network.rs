use std::{collections::HashSet, fs::read_to_string, iter::once, path::PathBuf};

use foray_data_model::node::{PortData, PortType};
use foray_graph::graph::{Graph, PortRef};
use iced::keyboard::Modifiers;
use indexmap::IndexMap;
use log::warn;
use serde::{Deserialize, Serialize};

use crate::{
    interface::node_canvas,
    math::Point,
    node_instance::{ForayNodeInstance, NodeStatus},
    workspace::Action,
};

type UndoStash = Vec<(
    Graph<ForayNodeInstance, PortType, PortData>,
    IndexMap<u32, Point>,
)>;

#[derive(Serialize, Deserialize, Default)]
pub struct Network {
    //// Persistant data
    pub graph: Graph<ForayNodeInstance, PortType, PortData>,
    pub shapes: node_canvas::node_canvas::State,
    //// Runtime data
    #[serde(skip)]
    pub file: Option<PathBuf>,
    #[serde(skip)]
    pub selected_shapes: HashSet<u32>,
    /// Nodes that are waiting for dependencies before executing
    /// TODO: make these cancleable
    #[serde(skip)]
    pub queued_nodes: HashSet<u32>,
    //#[serde(skip)]
    //pub compute_task_handles: HashMap<u32, iced::task::Handle>,
    #[serde(skip)]
    pub undo_stack: UndoStash,
    #[serde(skip)]
    pub redo_stack: UndoStash,
    #[serde(skip)]
    pub unsaved_changes: bool,
}
#[derive(Debug)]
pub enum NetworkLoadError {
    FileNotFound,
    CouldNotParse,
}

impl Network {
    pub fn load_network(path: &PathBuf) -> Result<Self, NetworkLoadError> {
        match read_to_string(path).map(|s| ron::from_str::<Network>(&s)) {
            Ok(Ok(mut network)) => {
                network.file = Some(path.clone());
                Ok(network)
            }
            Ok(Err(e)) => {
                warn!("Could not open file {path:?}\n{e}\nusing default network");
                Err(NetworkLoadError::FileNotFound)
            }
            Err(e) => {
                warn!("Could not parse file {path:?}\n{e}\nusing default network");
                warn!("creating default file");
                Err(NetworkLoadError::CouldNotParse)
            }
        }
    }

    /// Add an edge from input to output, removing existing connected input edge if present
    pub fn add_edge(&mut self, input: &PortRef, output: &PortRef) {
        self.stash_state();
        self.graph.remove_edge(input);
        self.graph.add_edge_from_ref(output, input);
    }

    /// Stash current app state, reset the redo stack, and mark unsaved changes
    pub fn stash_state(&mut self) {
        self.unsaved_changes = true;
        let mut graph_snap_shot = self.graph.clone();
        // We don't want to stash any node.status "running" values
        let running_nodes: Vec<_> = graph_snap_shot
            .nodes_ref()
            .into_iter()
            .filter(|nx| {
                matches!(
                    graph_snap_shot.get_node(*nx).status,
                    NodeStatus::Running { .. }
                )
            })
            .collect();
        for nx in running_nodes {
            graph_snap_shot.get_mut_node(nx).status = NodeStatus::Idle;
        }

        self.undo_stack
            .push((graph_snap_shot, self.shapes.shape_positions.clone()));

        // Don't let the stack get too big
        self.undo_stack.truncate(10);

        self.redo_stack.clear();
    }

    pub fn remove_edge(&mut self, port: PortRef) {
        self.stash_state();
        self.graph.remove_edge(&port);
    }

    pub(crate) fn select_node(
        &mut self,
        nx: u32,
        modifiers: Modifiers,
        cursor_position: Point,
    ) -> Action {
        self.selected_shapes = if modifiers.command() {
            //// Create new nodes on Command + Click
            self.stash_state();
            let selected_shapes = if self.selected_shapes.contains(&nx) {
                // If clicked node is already selected, copy all selected nodes,
                self.selected_shapes.clone()
            } else {
                // Otherwise, only copy the clicked node
                [nx].into()
            };
            selected_shapes
                .iter()
                .map(|id| {
                    let pos = self.shapes.shape_positions[id] + [5., 5.].into();
                    let new_node = self.graph.get_node(*id).template.clone().into();
                    // *Mutably* add new node to graph
                    let new_id = self.graph.node(new_node);
                    // *Mutably* add new position
                    self.shapes.shape_positions.insert(new_id, pos);
                    new_id
                })
                .collect()
        } else if modifiers.shift() {
            //// Select Multiple nodes if shift is held
            self.selected_shapes
                .clone()
                .into_iter()
                .chain(once(nx))
                .collect()
        } else if !self.selected_shapes.contains(&nx) {
            //// Select Single Node if an unselected node is clicked
            [nx].into()
        } else {
            //// Otherwise keep selection the same
            self.selected_shapes.clone()
        };

        let offsets = self
            .selected_shapes
            .iter()
            .map(|id| {
                (
                    *id,
                    (self.shapes.shape_positions[id]
                        - (cursor_position + self.shapes.camera.position)),
                )
            })
            .collect();

        //// Move selected shape to the top
        self.shapes.shape_positions.move_index(
            self.shapes
                .shape_positions
                .get_index_of(&nx)
                .expect("id exists"),
            0,
        );
        //// Start Drag
        Action::DragNode(offsets)
    }

    pub fn any_nodes_running(&self) -> bool {
        self.graph
            .nodes_ref()
            .into_iter()
            .map(|nx| self.graph.get_node(nx))
            .any(|node| matches!(node.status, NodeStatus::Running { .. }))
    }
}
