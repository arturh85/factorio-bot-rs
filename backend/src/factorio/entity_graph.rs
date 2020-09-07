use crate::num_traits::FromPrimitive;
use crate::types::{Direction, EntityType, FactorioEntity, Position};
use petgraph::graph::NodeIndex;
use petgraph::stable_graph::StableGraph;
use std::str::FromStr;

#[derive(Default, Clone)]
pub struct EntityNode {
    pub label: String,
    pub direction: Direction,
    pub entity_type: EntityType,
    pub entity: FactorioEntity,
    pub miner_ore: Option<String>,
}

impl std::fmt::Display for EntityNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.label)?;
        Ok(())
    }
}
impl std::fmt::Debug for EntityNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.label)?;
        Ok(())
    }
}

impl EntityNode {
    pub fn new(entity: FactorioEntity, miner_ore: Option<String>) -> EntityNode {
        let direction = Direction::from_u8(entity.direction).unwrap();
        let entity_type = EntityType::from_str(&entity.entity_type).unwrap();
        EntityNode {
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

pub type EntityGraph = StableGraph<EntityNode, f64>;

pub fn entity_node_at(entity_graph: &EntityGraph, position: &Position) -> Option<NodeIndex> {
    entity_graph.node_indices().find(|i| {
        if let Some(f) = entity_graph.node_weight(*i) {
            return f.entity.bounding_box.contains(&position);
        }
        false
    })
}
