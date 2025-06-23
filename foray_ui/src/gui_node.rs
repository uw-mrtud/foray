use std::sync::{Arc, RwLock, RwLockReadGuard};

use iced::{widget::text, Element};

use crate::{
    app::Message,
    graph::Graph,
    nodes::{
        port::{PortData, PortType},
        status::NodeStatus,
        NodeData,
    },
    StableMap,
};

pub trait GUINode: derive_more::Debug {
    //TODO make this more understandable. clearer distinction between graph and gui?
    // split out port names, and compute logic?
    //fn network_node(&self) -> GraphNode<PortType, PortData>;

    //TODO: Port validation logic? here, or handled at the portType level?
    //TODO: conversion logic?

    fn name(&self) -> String;

    fn view(
        &'_ self,
        _id: u32,
        _input_data: StableMap<String, PortDataContainer>,
    ) -> Element<'_, Message> {
        text("default").into()
    }
    fn node_size(&self) -> iced::Size;

    fn config_view(
        &self,
        _id: u32,
        _input_data: StableMap<String, PortDataContainer>,
    ) -> Option<Element<'_, Message>> {
        None
    }
}

pub type PortDataReference<'a> = RwLockReadGuard<'a, PortData>;
pub type PortDataContainer = Arc<RwLock<PortData>>;
pub type GuiGraph = Graph<NodeData, PortType, PortData>;

impl GuiGraph {
    pub fn running_nodes(&self) -> Vec<&NodeData> {
        self.nodes_ref()
            .into_iter()
            .map(|nx| self.get_node(nx))
            .filter(|node| matches!(node.status, NodeStatus::Running(..)))
            .collect()
    }
}
