use foray_data_model::node::{PortData, PortType};

use crate::graph::{Graph, GraphNode};

pub fn execute_graph<T: GraphNode<PortType, PortData>>(g: &mut Graph<T, PortType, PortData>)
where
    T: Clone,
{
    // Propogate values
    for nx in g.topological_sort() {
        let (node, input_guarded) = g.get_compute(nx);
        let (_, output) = Graph::compute_node(nx, node, input_guarded);
        g.update_wire_data(nx, output.unwrap());
    }
}
