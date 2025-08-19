use std::{error::Error, path::PathBuf};

use foray_graph::graph::Graph;
use log::trace;

use crate::{network::Network, python_env};

pub fn run_headless(network_path: PathBuf) -> Result<(), Box<dyn Error>> {
    let venv_dir = network_path
        .parent()
        .expect("Network should be a file")
        .join("../.venv");

    python_env::setup_python(venv_dir);

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
