use std::{collections::HashSet, fs::read_to_string, path::PathBuf};

use data_model::node::{NodeTemplate, PortData, PortType};
use log::{error, warn};
use serde::{Deserialize, Serialize};

use crate::{
    graph::{Graph, NodeIndex},
    project::Project,
    // math::Point,
    // nodes::{
    //     NodeData, NodeTemplate,
    //     port::{PortData, PortType},
    //     status::NodeStatus,
    // },
    // project::Project, // widget::{shapes::ShapeId, workspace},
};

#[derive(Serialize, Deserialize, Default)]
pub struct Network {
    //// Persistant data
    pub graph: Graph<NodeTemplate, PortType, PortData>,
    //// Runtime data
    #[serde(skip)]
    pub file: Option<PathBuf>,
    /// Nodes that are waiting for dependencies before executing
    /// TODO: make these canceleable
    #[serde(skip)]
    pub queued_nodes: HashSet<NodeIndex>,
    //#[serde(skip)]
    //pub compute_task_handles: HashMap<u32, iced::task::Handle>,
}
pub enum NetworkLoadError {
    FileNotFound,
    CouldNotParse,
}

impl Network {
    pub fn load_network(path: &PathBuf, projects: &[Project]) -> Result<Self, NetworkLoadError> {
        match read_to_string(path).map(|s| ron::from_str::<Self>(&s)) {
            Ok(Ok(mut network)) => {
                network.file = Some(path.clone());
                let node_ids = network.graph.nodes_ref();
                node_ids.into_iter().for_each(|nx| {
                    let py_node = &mut network.graph.get_mut_node(nx);
                    // NodeTemplate::RustNode(ref _rust_node) => {}
                    // NodeTemplate::PyNode(ref mut py_node) => {
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
                        error!("Could not find source file for node \n{py_node:?}");
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

    // /// Add an edge from input to output, removing existing connected input edge if present
    // pub fn add_edge(&mut self, input: &PortRef, output: &PortRef) {
    //     self.graph.remove_edge(input);
    //     self.graph.add_edge_from_ref(output, input);
    // }
    //
    // pub fn remove_edge(&mut self, port: crate::graph::PortRef) {
    //     self.graph.remove_edge(&port);
    // }
}
