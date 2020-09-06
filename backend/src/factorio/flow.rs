use crate::types::Position;
use petgraph::stable_graph::StableGraph;

#[derive(Debug, Clone)]
pub enum FlowType {}

#[derive(Default, Clone)]
pub struct FlowNode {
    pub label: String,
    pub entity_name: String,
    pub entity_type: String,
    pub position: Position,
    pub data: Option<FlowType>,
}

impl FlowNode {
    pub fn new() -> FlowNode {
        FlowNode {
            ..Default::default()
        }
    }
}

pub type FlowGraph = StableGraph<FlowNode, f64>;
