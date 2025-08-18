use std::{error::Error, path::PathBuf};

use foray_graph::graph::Graph;
use log::trace;

use crate::{config::Config, network::Network};

pub fn run_headless(network_path: PathBuf) -> Result<(), Box<dyn Error>> {
    let config = Config::read_config();
    config.setup_environment();
    let projects = config.read_projects();
    trace!(
        "Configured Python Projects: {:?}",
        projects
            .iter()
            .map(|p| p.absolute_path.clone())
            .collect::<Vec<_>>()
    );

    let network = match Network::load_network(&network_path) {
        Ok(n) => n,
        Err(e) => panic!("{:?}", e),
    };
    let mut graph = network.graph;

    // Propogate values
    for nx in graph.topological_sort() {
        trace!("Executing node {nx}");
        let (node, input_guarded) = graph.get_compute(nx);
        let (_, output) = Graph::compute_node(nx, node, input_guarded);
        graph.update_wire_data(nx, output.unwrap());
    }

    Ok(())
}
