use crate::types::{FactorioEntity, Position};
use petgraph::graph::NodeIndex;
use petgraph::stable_graph::StableGraph;

#[derive(Debug, Clone)]
pub enum FlowType {}

#[derive(Default, Clone)]
pub struct FlowNode {
    pub label: String,
    pub entity: FactorioEntity,
    pub data: Option<FlowType>,
}

impl std::fmt::Display for FlowNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.label)?;
        Ok(())
    }
}
impl std::fmt::Debug for FlowNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.label)?;
        Ok(())
    }
}

impl FlowNode {
    pub fn new(entity: FactorioEntity) -> FlowNode {
        FlowNode {
            label: format!("{} at {}", entity.entity_type, entity.position),
            entity,
            ..Default::default()
        }
    }
}

pub type FlowGraph = StableGraph<FlowNode, f64>;

pub fn flow_node_at(flow: &FlowGraph, position: &Position) -> Option<NodeIndex> {
    flow.node_indices().find(|i| {
        if let Some(f) = flow.node_weight(*i) {
            return f.entity.bounding_box.contains(&position);
        }
        false
    })
}
