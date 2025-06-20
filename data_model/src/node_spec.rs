use std::sync::{Arc, RwLock};

use serde::Deserialize;

use crate::node::{Dict, NodeError, PortData};

pub type PortName = String;
pub type WireDataContainer<T> = Arc<RwLock<T>>;

pub trait GraphNode<NodeData, PortType, WireData>
where
    PortType: Clone,
{
    fn inputs(&self) -> Dict<PortName, PortType>;
    fn outputs(&self) -> Dict<PortName, PortType>;
    fn compute(
        self,
        inputs: Dict<PortName, WireDataContainer<WireData>>,
    ) -> Result<(Dict<PortName, WireData>, NodeData), NodeError>;
}
