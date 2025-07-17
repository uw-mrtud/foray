use foray_data_model::node::{PortData, PortType};

use crate::{graph::Graph, node_instance::ForayNodeInstance};

pub fn execute_graph(g: &mut Graph<ForayNodeInstance, PortType, PortData>) {
    // Propogate values
    for nx in g.topological_sort() {
        let (node, input_guarded) = g.get_compute(nx);
        let (_, output) = Graph::compute_node(nx, node, input_guarded);
        g.update_wire_data(nx, output.unwrap());
    }
}
