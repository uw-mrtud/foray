use std::{collections::HashSet, fs::read_to_string, iter::once, path::PathBuf};

use iced::keyboard::Modifiers;
use indexmap::IndexMap;
use log::{error, warn};
use serde::{Deserialize, Serialize};

use crate::{
    app::Action,
    graph::{Graph, PortRef},
    gui_node::GuiGraph,
    math::Point,
    nodes::{
        port::{PortData, PortType},
        status::NodeStatus,
        NodeData, NodeTemplate,
    },
    project::Project,
    widget::{shapes::ShapeId, workspace},
};

type UndoStash = Vec<(
    Graph<NodeData, PortType, PortData>,
    IndexMap<ShapeId, Point>,
)>;

#[derive(Serialize, Deserialize, Default)]
pub struct Network {
    //// Persistant data
    pub graph: GuiGraph,
    pub shapes: workspace::State,
    //// Runtime data
    #[serde(skip)]
    pub file: Option<PathBuf>,
    #[serde(skip)]
    pub selected_shapes: HashSet<ShapeId>,
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
pub enum NetworkLoadError {
    FileNotFound,
    CouldNotParse,
}

impl Network {
    pub fn load_network(path: &PathBuf, projects: &[Project]) -> Result<Self, NetworkLoadError> {
        match read_to_string(path).map(|s| ron::from_str::<Network>(&s)) {
            Ok(Ok(mut network)) => {
                network.file = Some(path.clone());
                let node_ids = network.graph.nodes_ref();
                node_ids.into_iter().for_each(|nx| {
                    match &mut network.graph.get_mut_node(nx).template {
                        NodeTemplate::RustNode(ref _rust_node) => {}
                        NodeTemplate::PyNode(ref mut py_node) => {
                            // Resolve the absolute path, given the nodes we know are
                            // accessible.
                            // Currently We just take the first one found, but more complex
                            // resolution could be added
                            let found_path = projects
                                .iter()
                                // Calculate potential node source path
                                .map(|project| {
                                    py_node
                                        .relative_path
                                        .to_logical_path(project.absolute_path.clone())
                                })
                                // Pick the first path that exists
                                .find_map(|path| {
                                    if path.is_file() {
                                        Some(path.clone())
                                    } else {
                                        None
                                    }
                                });
                            if let Some(path) = found_path {
                                py_node.absolute_path = path.to_path_buf();
                            } else {
                                error!("Could not find source file for node \n{py_node}");
                            }
                        }
                    }
                });
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
                    NodeStatus::Running(..)
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

    pub fn remove_edge(&mut self, port: crate::graph::PortRef) {
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
                    let new_node = self.graph.get_node(*id).template.duplicate().into();
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
}
