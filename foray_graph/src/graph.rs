use std::{collections::HashMap, sync::Arc};

use derive_more::Display;
use foray_py::err::PyNodeConfigError;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

// use crate::nodes::status::NodeError;

use foray_data_model::{
    WireDataContainer,
    node::{Dict, NodeError},
};

pub type PortName = String;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Display)]
pub enum ForayNodeError {
    PyNodeConifgError(PyNodeConfigError),
    NodeError(NodeError),
}

pub trait GraphNode<PortType, WireData>
where
    PortType: Clone,
{
    fn inputs(&self) -> Dict<PortName, PortType>;
    fn outputs(&self) -> Dict<PortName, PortType>;
    fn compute(
        self,
        populated_inputs: Dict<PortName, WireDataContainer<WireData>>,
        // parameters: Dict<PortName, WireData>,
    ) -> Result<Dict<PortName, WireData>, ForayNodeError>;
}

pub type NodeIndex = u32;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct PortRef {
    pub node: NodeIndex,
    pub name: PortName,
    pub io: IO,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum IO {
    In,
    Out,
}

type Edge = (PortRef, PortRef);

#[derive(Debug, Serialize, Deserialize)]
pub struct Graph<NodeData, PortType, WireData>
where
    NodeData: GraphNode<PortType, WireData>,
    PortType: Clone,
    WireData: std::fmt::Debug,
{
    nodes: Dict<NodeIndex, NodeData>,
    edges: Vec<Edge>,
    #[serde(skip, default = "default_wire_data")]
    wire_data: HashMap<(NodeIndex, PortName), WireDataContainer<WireData>>,
    next_id: NodeIndex,
    #[serde(skip)]
    phantom: std::marker::PhantomData<PortType>,
}

impl<N: GraphNode<P, W>, P: Clone, W: std::fmt::Debug> Graph<N, P, W> {}

impl<NodeData: Clone, PortType: Clone, WireData> Clone for Graph<NodeData, PortType, WireData>
where
    NodeData: GraphNode<PortType, WireData>,
    PortType: Clone,
    WireData: std::fmt::Debug,
{
    fn clone(&self) -> Self {
        Self {
            nodes: self.nodes.clone(),
            edges: self.edges.clone(),
            wire_data: Default::default(),
            next_id: self.next_id,
            phantom: self.phantom,
        }
    }
}

fn default_wire_data<K, V>() -> HashMap<K, V> {
    HashMap::new()
}
impl<NodeData, PortType, WireData> Graph<NodeData, PortType, WireData>
where
    NodeData: GraphNode<PortType, WireData> + Clone,
    PortType: Clone,
    WireData: std::fmt::Debug,
{
    pub fn new() -> Self {
        Self {
            nodes: Dict::new(),
            edges: vec![],
            wire_data: HashMap::new(),
            next_id: 0,
            phantom: std::marker::PhantomData,
        }
    }

    /// Add a new node to the graph, returns the node's index
    pub fn node(&mut self, node: NodeData) -> NodeIndex {
        let id = self.next_id;
        self.nodes.insert(id, node);
        self.next_id += 1;
        id
    }

    /// Remove a node and all edges associated with it
    pub fn delete_node(&mut self, id: NodeIndex) {
        self.nodes.remove(&id);
        self.edges
            .retain(|(from, to)| from.node != id && to.node != id)
    }
    /// Get the node value at a given index
    /// panics if index is not valid!
    /// Use the index returned from `add_node` to ensure it exists
    pub fn get_node(&self, nx: NodeIndex) -> &NodeData {
        &self.nodes[&nx]
    }

    /// Get a mutable reference  to a node value at a given index
    /// panics if index is not valid
    /// Use the index returned from `add_node` to ensure it exists
    pub fn get_mut_node(&mut self, nx: NodeIndex) -> &mut NodeData {
        self.nodes.get_mut(&nx).unwrap()
    }

    pub fn get_output_data(&self, nx: &NodeIndex) -> Dict<String, WireDataContainer<WireData>> {
        self.get_node(*nx)
            .outputs()
            .keys()
            .filter_map(|port_name| {
                self.wire_data
                    .get(&(*nx, port_name.clone()))
                    .map(|data| (port_name.clone(), data.clone()))
            })
            .collect()
    }

    pub fn get_input_data(&self, nx: &NodeIndex) -> Dict<String, WireDataContainer<WireData>> {
        self.get_node(*nx)
            .inputs()
            .keys()
            .filter_map(|port_name| {
                self.get_parent(nx, port_name.clone()).map(|out_port| {
                    self.wire_data
                        .get(&(out_port.node, out_port.name))
                        .map(|data| (port_name.clone(), data.clone()))
                })
            })
            .collect::<Option<Dict<_, _>>>()
            .unwrap_or([].into())
    }

    pub fn get_input_data_mapped(
        &self,
        nx: &NodeIndex,
    ) -> (
        Dict<String, String>,
        Dict<String, WireDataContainer<WireData>>,
    ) {
        let inputs = self.get_node(*nx).inputs();

        let data_with_duplicates = inputs
            .keys()
            .filter_map(|port_name| {
                self.get_parent(nx, port_name.clone()).map(|out_port| {
                    self.wire_data
                        .get(&(out_port.node, out_port.name))
                        .map(|data| (port_name.clone(), data))
                })
            })
            .collect::<Option<Dict<_, _>>>()
            .unwrap_or([].into());

        let port_matches: Dict<String, String> = data_with_duplicates
            .iter()
            .combinations(2)
            .filter(|v| v[0].0 != v[1].0)
            .flat_map(|v| {
                let (a_key, a_value) = v[0];
                let (b_key, b_value) = v[1];
                if Arc::ptr_eq(a_value, b_value) {
                    println!("{a_key} points to the same Arc as {b_key}");
                    vec![
                        (a_key.clone(), a_key.clone()),
                        (b_key.clone(), a_key.clone()),
                    ]
                } else {
                    vec![(a_key.clone(), a_key.clone())]
                }
            })
            .collect();
        let data = port_matches
            .keys()
            .map(|k| (k.clone(), Arc::clone(data_with_duplicates[k])))
            .collect();

        (port_matches, data)
    }

    /// Get a list of node indices
    pub fn nodes_ref(&self) -> Vec<NodeIndex> {
        self.nodes.keys().copied().collect()
    }

    /// Set the node value of an existing node
    pub fn set_node_data(&mut self, nx: NodeIndex, value: NodeData) {
        *self.nodes.get_mut(&nx).unwrap() = value;
    }

    pub fn update_wire_data(&mut self, nx: NodeIndex, outputs: Dict<PortName, WireData>) {
        for (port_name, wire_data) in outputs.into_iter() {
            self.wire_data
                .insert((nx, port_name), Arc::new(wire_data.into()));
        }
    }
    pub fn clear_outputs(&mut self, nx: NodeIndex) {
        self.get_node(nx).outputs().keys().for_each(|output_name| {
            self.wire_data.remove(&(nx, output_name.clone()));
        });
    }

    pub fn get_wire_data(
        &self,
        nx: &NodeIndex,
        port_name: &str,
    ) -> Option<&WireDataContainer<WireData>> {
        self.wire_data.get(&(*nx, port_name.into()))
    }

    /// Create a connection between two port references
    pub fn add_edge_from_ref(&mut self, from: &PortRef, to: &PortRef) {
        assert!(from.io == IO::Out);
        assert!(to.io == IO::In);
        self.connect((from.node, from.name.clone()), (to.node, to.name.clone()));
    }
    /// Create a connection between two ports
    pub fn connect(
        &mut self,
        from: (NodeIndex, impl Into<PortName>),
        to: (NodeIndex, impl Into<PortName>),
    ) {
        let from = PortRef {
            node: from.0,
            name: from.1.into(),
            io: IO::Out,
        };
        let to = PortRef {
            node: to.0,
            name: to.1.into(),
            io: IO::In,
        };

        //TODO: check for compatiablity, or if the edge alread exists
        // warn if already exists, panic/return result if incompatabible
        self.edges.push((from, to));
    }

    /// Remove any edges associated with the given port
    pub fn remove_edge(&mut self, port: &PortRef) {
        self.edges.retain(|(from, to)| port != from && port != to)
    }

    pub fn get_parent(&self, nx: &NodeIndex, in_port: PortName) -> Option<PortRef> {
        self.edges
            .iter()
            .find(|(_from, to)| to.node == *nx && to.name == in_port)
            .map(|(from, _to)| from.clone())
    }

    /// Find the index of the port based on the order defined in the `GraphNode`
    /// panics if `port` is not valid
    pub fn port_index(&self, port: &PortRef) -> usize {
        match port.io {
            IO::In => self
                .get_node(port.node)
                .inputs()
                .iter()
                .position(|n| *n.0 == *port.name)
                .unwrap_or_else(|| {
                    panic!("PortId must have valid input node index and port id {port:?}",)
                }),
            IO::Out => self
                .get_node(port.node)
                .outputs()
                .iter()
                .position(|n| *n.0 == *port.name)
                .unwrap_or_else(|| {
                    panic!("PortId must have valid input node index and port id {port:?}",)
                }),
        }
    }
    /// Find a nodes direct parents and the associated labels
    pub fn incoming_edges(&self, nx: &NodeIndex) -> Vec<(PortRef, PortRef)> {
        self.edges
            .iter()
            .filter_map(|(from, to)| {
                if to.node == *nx {
                    Some((from.clone(), to.clone()))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Find the edges that that originate at `nx`
    pub fn outgoing_edges(&self, nx: &NodeIndex) -> Vec<PortRef> {
        self.edges
            .iter()
            .filter_map(|(from, to)| {
                if from.node == *nx {
                    Some(to.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Topological sort using Kahn's algorithm
    /// returns a list of NodeIndices
    pub fn topological_sort(&self) -> Vec<NodeIndex> {
        let mut sorted = vec![];
        let mut working_edges = self.edges.clone();

        let mut no_incoming: Vec<_> = self
            .nodes
            .keys()
            .filter(|nx| !Self::has_incoming(nx, &working_edges))
            .copied()
            .collect();

        while let Some(nx) = no_incoming.pop() {
            sorted.push(nx);
            while let Some(ex) = Self::next_connected_edge(&nx, &working_edges) {
                let edge = working_edges.swap_remove(ex);
                let mx = edge.1.node;
                if !Self::has_incoming(&mx, &working_edges) {
                    no_incoming.push(mx);
                }
            }
        }
        if working_edges.is_empty() {
            sorted
        } else {
            panic!("graph has cycles!")
        }
    }

    /// Determine if a node has any incoming connections
    fn has_incoming(nx: &NodeIndex, edges: &[Edge]) -> bool {
        edges.iter().any(|(_from, to)| to.node == *nx)
    }

    /// Find the index of `edges` corresponding to the first
    /// connection starting from `nx` (if it exists)
    fn next_connected_edge(nx: &NodeIndex, edges: &[Edge]) -> Option<usize> {
        edges.iter().position(|(from, _to)| from.node == *nx)
    }

    pub fn get_compute(
        &self,
        nx: NodeIndex,
    ) -> (NodeData, Dict<String, WireDataContainer<WireData>>) {
        let node = self.get_node(nx);
        let wire_data = self.get_input_data(&nx);
        (node.clone(), wire_data)
    }

    #[allow(clippy::type_complexity)]
    pub fn compute_node(
        nx: NodeIndex,
        node: NodeData,
        input_guarded: Dict<String, WireDataContainer<WireData>>,
    ) -> (NodeIndex, Result<Dict<String, WireData>, ForayNodeError>) {
        let output = { node.compute(input_guarded) };

        (nx, output)
    }
    pub async fn async_compute(
        nx: NodeIndex,
        node: NodeData,
        input_guarded: Dict<String, WireDataContainer<WireData>>,
    ) -> (NodeIndex, Result<Dict<String, WireData>, ForayNodeError>) {
        Self::compute_node(nx, node, input_guarded)
    }

    /// get all nodes that have no parents
    pub fn get_roots(&self) -> Vec<NodeIndex> {
        self.nodes
            .keys()
            .filter(|nx| self.incoming_edges(nx).is_empty())
            .copied()
            .collect()
    }
}

impl<NodeData, PortType, WireData> Default for Graph<NodeData, PortType, WireData>
where
    NodeData: GraphNode<PortType, WireData> + Clone,
    PortType: Clone,
    WireData: std::fmt::Debug,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[derive(Clone, Debug)]
    struct IdentityNode {}
    #[derive(Clone, Debug)]
    struct ConstantNode {
        value: u32,
    }

    #[derive(Clone, Debug)]
    enum Node {
        Identity(IdentityNode),
        Constant(ConstantNode),
    }

    impl GraphNode<(), u32> for Node {
        fn inputs(&self) -> Dict<String, ()> {
            match self {
                Node::Identity(_node) => [("in".to_string(), ())].into(),
                Node::Constant(_node) => [].into(),
            }
        }

        fn outputs(&self) -> Dict<String, ()> {
            match self {
                Node::Identity(_node) => [("out".to_string(), ())].into(),
                Node::Constant(_node) => [("out".to_string(), ())].into(),
            }
        }

        fn compute(
            self,
            inputs: Dict<String, WireDataContainer<u32>>,
        ) -> Result<Dict<String, u32>, ForayNodeError> {
            dbg!(&inputs);
            dbg!(&self);
            Ok(match &self {
                Node::Identity(_node) => {
                    [("out".to_string(), *inputs["in"].read().unwrap())].into()
                }

                Node::Constant(node) => [("out".to_string(), node.value)].into(),
            })
        }
    }

    #[test]
    fn sort() {
        let mut g: Graph<Node, (), u32> = Graph::new();

        let n8 = g.node(Node::Identity(IdentityNode {}));
        let n7 = g.node(Node::Identity(IdentityNode {}));
        let n6 = g.node(Node::Identity(IdentityNode {}));
        let n5 = g.node(Node::Identity(IdentityNode {}));
        let n4 = g.node(Node::Identity(IdentityNode {}));
        let n3 = g.node(Node::Identity(IdentityNode {}));
        let n2 = g.node(Node::Identity(IdentityNode {}));
        let n1 = g.node(Node::Identity(IdentityNode {}));

        g.connect((n1, "out"), (n3, "in"));
        g.connect((n1, "out"), (n2, "in"));
        g.connect((n3, "out"), (n4, "in"));
        g.connect((n4, "out"), (n5, "in"));
        g.connect((n5, "out"), (n6, "in"));
        g.connect((n6, "out"), (n7, "in"));
        g.connect((n7, "out"), (n8, "in"));
        assert_eq!(g.topological_sort(), vec![7, 6, 5, 4, 3, 2, 1, 0]);
    }

    #[test]
    fn process() {
        let mut g: Graph<Node, (), u32> = Graph::new();

        let n1 = g.node(Node::Constant(ConstantNode { value: 7 }));
        let n2 = g.node(Node::Identity(IdentityNode {}));
        let n3 = g.node(Node::Identity(IdentityNode {}));
        let n4 = g.node(Node::Identity(IdentityNode {}));

        g.connect((n1, "out"), (n3, "in"));
        g.connect((n1, "out"), (n2, "in"));
        g.connect((n3, "out"), (n4, "in"));

        // Propogate values
        for nx in g.topological_sort() {
            let (node, input_guarded) = g.get_compute(nx);
            let (_, output) = Graph::compute_node(nx, node, input_guarded);
            g.update_wire_data(nx, output.unwrap());
        }

        assert_eq!(*g.get_wire_data(&n1, "out").unwrap().read().unwrap(), 7);
        assert_eq!(*g.get_wire_data(&n2, "out").unwrap().read().unwrap(), 7);
        assert_eq!(*g.get_wire_data(&n3, "out").unwrap().read().unwrap(), 7);
    }
    //TODO: test unconnected nodes, making sure we don't try to run nodes without the necessary
    //inputs
}
