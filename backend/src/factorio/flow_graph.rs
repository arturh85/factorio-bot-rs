use crate::factorio::entity_graph::EntityGraph;
use crate::num_traits::FromPrimitive;
use crate::types::{Direction, EntityType, FactorioEntity, Position};
use petgraph::graph::NodeIndex;
use petgraph::stable_graph::StableGraph;
use petgraph::visit::EdgeRef;
use std::str::FromStr;

#[derive(Default, Clone)]
pub struct FlowNode {
    pub label: String,
    pub direction: Direction,
    pub entity_type: EntityType,
    pub entity: FactorioEntity,
    pub miner_ore: Option<String>,
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
    pub fn new(entity: FactorioEntity, miner_ore: Option<String>) -> FlowNode {
        let direction = Direction::from_u8(entity.direction).unwrap();
        let entity_type = EntityType::from_str(&entity.entity_type).unwrap();
        FlowNode {
            label: format!(
                "{}{} at {}",
                if let Some(miner_ore) = &miner_ore {
                    format!("{}: ", miner_ore)
                } else {
                    String::new()
                },
                entity.entity_type,
                entity.position
            ),
            direction,
            miner_ore,
            entity,
            entity_type,
        }
    }
}

pub type FlowGraph = StableGraph<FlowNode, f64>;

pub fn flow_node_at(graph: &FlowGraph, position: &Position) -> Option<NodeIndex> {
    graph.node_indices().find(|i| {
        if let Some(f) = graph.node_weight(*i) {
            return f.entity.bounding_box.contains(&position);
        }
        false
    })
}

pub fn build_flow_graph(entity_graph: &EntityGraph) -> FlowGraph {
    let mut graph = FlowGraph::new();
    for node_index in entity_graph.node_indices() {
        let node = entity_graph.node_weight(node_index).unwrap();
        if node.entity_type == EntityType::MiningDrill && node.miner_ore.is_some() {
            let _root = graph.add_node(FlowNode::new(node.entity.clone(), node.miner_ore.clone()));
            if let Some(next) = entity_graph
                .edges_directed(node_index, petgraph::Direction::Outgoing)
                .next()
            {
                let _entity_node = entity_graph.node_weight(next.target()).unwrap();
            }
        }
    }
    graph
}
