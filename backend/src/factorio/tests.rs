use crate::factorio::entity_graph::EntityGraph;
#[cfg(test)]
use crate::types::{FactorioEntity, FactorioEntityPrototype};
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::Arc;

pub fn entity_graph_from(entities: Vec<FactorioEntity>) -> anyhow::Result<EntityGraph> {
    let prototypes = fixture_entity_prototypes();
    let graph = EntityGraph::new(Arc::new(prototypes), Arc::new(DashMap::new()));
    graph.add(entities, None)?;
    graph.connect()?;
    Ok(graph)
}

pub fn fixture_entity_prototypes() -> DashMap<String, FactorioEntityPrototype> {
    let prototypes: HashMap<String, FactorioEntityPrototype> =
        serde_json::from_str(include_str!("../../tests/entity-prototype-fixtures.json"))
            .expect("failed to parse fixture");
    let dashmap: DashMap<String, FactorioEntityPrototype> = DashMap::new();
    for foo in prototypes {
        dashmap.insert(foo.0, foo.1);
    }
    dashmap
}
