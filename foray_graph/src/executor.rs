use data_model::node::{NodeTemplate, PortData, PortType};

use crate::graph::Graph;

pub fn execute_graph(g: &mut Graph<NodeTemplate, PortType, PortData>) {
    // Propogate values
    for nx in g.topological_sort() {
        let (node, input_guarded) = g.get_compute(nx);
        let (_, output) = Graph::compute_node(nx, node, input_guarded);
        g.update_wire_data(nx, output.unwrap().0);
    }
}
