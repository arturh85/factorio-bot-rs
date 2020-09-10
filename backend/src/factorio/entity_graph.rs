use crate::factorio::util::{move_position, rect_fields, rect_floor};
use crate::num_traits::FromPrimitive;
use crate::types::{Direction, EntityName, EntityType, FactorioEntity, Pos, Position};
use dashmap::lock::{RwLock, RwLockReadGuard};
use petgraph::graph::{EdgeIndex, NodeIndex};
use petgraph::stable_graph::StableGraph;
use petgraph::visit::{Bfs, EdgeRef};
use std::ops::Deref;
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

pub type EntityGraphInner = StableGraph<EntityNode, f64>;

pub struct EntityGraph {
    inner: RwLock<EntityGraphInner>,
    // quad_tree: Quadtree<i32, NodeIndex>,
}

impl EntityGraph {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        EntityGraph {
            inner: RwLock::new(EntityGraphInner::new()),
            // quad_tree: Quadtree::new(16),
        }
    }
    pub fn from<F>(vec: Vec<FactorioEntity>, check_for_resource: F) -> anyhow::Result<Self>
    where
        F: FnOnce(&str, Pos) -> bool + Copy,
    {
        let graph = EntityGraph::new();
        for e in vec {
            graph.add(&e, check_for_resource)?;
        }
        graph.connect()?;
        Ok(graph)
    }
    pub fn inner(&self) -> RwLockReadGuard<EntityGraphInner> {
        self.inner.read()
    }

    pub fn add<F>(&self, entity: &FactorioEntity, check_for_resource: F) -> anyhow::Result<()>
    where
        F: FnOnce(&str, Pos) -> bool + Copy,
    {
        let mut inner = self.inner.write();
        EntityGraph::_add(&mut inner, entity, check_for_resource)?;
        Ok(())
    }

    #[allow(clippy::ptr_arg)]
    pub fn add_multiple<F>(
        &self,
        entities: &Vec<FactorioEntity>,
        check_for_resource: F,
    ) -> anyhow::Result<()>
    where
        F: FnOnce(&str, Pos) -> bool + Copy,
    {
        let mut inner = self.inner.write();
        for entity in entities {
            EntityGraph::_add(&mut inner, entity, check_for_resource)?;
        }
        Ok(())
    }

    fn _add<F>(
        inner: &mut EntityGraphInner,
        entity: &FactorioEntity,
        check_for_resource: F,
    ) -> anyhow::Result<()>
    where
        F: FnOnce(&str, Pos) -> bool + Copy,
    {
        if let Ok(entity_type) = EntityType::from_str(&entity.entity_type) {
            match entity_type {
                EntityType::Furnace
                | EntityType::Inserter
                | EntityType::Boiler
                | EntityType::OffshorePump
                | EntityType::MiningDrill
                | EntityType::Container
                | EntityType::Splitter
                | EntityType::TransportBelt
                | EntityType::UndergroundBelt
                | EntityType::Pipe
                | EntityType::PipeToGround
                | EntityType::Assembler => {
                    if EntityGraph::node_at(&inner, &entity.position).is_none() {
                        let miner_ore = if entity_type == EntityType::MiningDrill {
                            let rect = rect_floor(&entity.bounding_box);
                            let mut miner_ore: Option<String> = None;
                            for resource in &[
                                EntityName::IronOre,
                                EntityName::CopperOre,
                                EntityName::Coal,
                                EntityName::Stone,
                                EntityName::UraniumOre,
                            ] {
                                let resource = resource.to_string();
                                let resource_found = rect_fields(&rect)
                                    .iter()
                                    .any(|p| check_for_resource(&resource, p.into()));
                                if resource_found {
                                    miner_ore = Some(resource);
                                    break;
                                }
                            }
                            if miner_ore.is_none() {
                                warn!("no ore found under miner {:?}", &entity);
                            }
                            miner_ore
                        } else {
                            None
                        };
                        let new_node = EntityNode::new(entity.clone(), miner_ore);
                        let _new_node_index = inner.add_node(new_node);
                        // self.quad_tree.insert(
                        //     AreaBuilder::default()
                        //         .anchor()
                        //         .dimensions()
                        //         .build()
                        //         .expect("Unexpected error in Area::contains_pt."),
                        //     new_node_index,
                        // );
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn incoming_nodes(graph: &EntityGraphInner, node_index: NodeIndex) -> Vec<&EntityNode> {
        graph
            .edges_directed(node_index, petgraph::Direction::Incoming)
            .map(|edge| graph.node_weight(edge.source()).unwrap())
            .collect()
    }
    fn incoming_weights(graph: &EntityGraphInner, node_index: NodeIndex) -> Vec<f64> {
        graph
            .edges_directed(node_index, petgraph::Direction::Incoming)
            .map(|edge| *edge.weight())
            .collect()
    }
    fn incoming_node_indexes(graph: &EntityGraphInner, node_index: NodeIndex) -> Vec<NodeIndex> {
        graph
            .edges_directed(node_index, petgraph::Direction::Incoming)
            .map(|edge| edge.source())
            .collect()
    }
    fn outgoing_nodes(graph: &EntityGraphInner, node_index: NodeIndex) -> Vec<&EntityNode> {
        graph
            .edges_directed(node_index, petgraph::Direction::Outgoing)
            .map(|edge| graph.node_weight(edge.target()).unwrap())
            .collect()
    }
    fn outgoing_weights(graph: &EntityGraphInner, node_index: NodeIndex) -> Vec<f64> {
        graph
            .edges_directed(node_index, petgraph::Direction::Outgoing)
            .map(|edge| *edge.weight())
            .collect()
    }
    fn outgoing_node_indexes(graph: &EntityGraphInner, node_index: NodeIndex) -> Vec<NodeIndex> {
        graph
            .edges_directed(node_index, petgraph::Direction::Outgoing)
            .map(|edge| edge.target())
            .collect()
    }

    pub fn condense(&self) -> EntityGraphInner {
        let mut graph = self.inner.read().clone();
        let mut roots: Vec<usize> = vec![];
        loop {
            let mut next_node: Option<NodeIndex> = None;
            for node_index in graph.externals(petgraph::Direction::Incoming) {
                if !roots.contains(&node_index.index()) {
                    roots.push(node_index.index());
                    next_node = Some(node_index);
                    break;
                }
            }
            if let Some(next_node) = next_node {
                let mut bfs = Bfs::new(&graph, next_node);
                while let Some(nx) = bfs.next(&graph) {
                    self.condense_walk(&mut graph, nx);
                }
            } else {
                break;
            }
        }
        graph
    }
    pub fn condense_walk(&self, graph: &mut EntityGraphInner, node_index: NodeIndex) {
        let node = graph.node_weight(node_index).unwrap();
        let incoming = EntityGraph::incoming_nodes(graph, node_index);
        let outgoing = EntityGraph::outgoing_nodes(graph, node_index);
        if incoming.len() == 1
            && outgoing.len() == 1
            && node.entity.name == incoming[0].entity.name
            && incoming[0].entity.name == outgoing[0].entity.name
        {
            let incoming = EntityGraph::incoming_node_indexes(graph, node_index)
                .pop()
                .unwrap();
            let outgoing = EntityGraph::outgoing_node_indexes(graph, node_index)
                .pop()
                .unwrap();
            let weight = EntityGraph::incoming_weights(graph, node_index)
                .pop()
                .unwrap()
                + EntityGraph::outgoing_weights(graph, node_index)
                    .pop()
                    .unwrap();

            graph.remove_edge(graph.find_edge(incoming, node_index).unwrap());
            graph.remove_edge(graph.find_edge(node_index, outgoing).unwrap());
            graph.add_edge(incoming, outgoing, weight);
            graph.remove_node(node_index);
        } else if incoming.len() == 2
            && outgoing.len() == 2
            && node.entity.name == incoming[0].entity.name
            && incoming[0].entity.name == outgoing[0].entity.name
        {
            let incoming = EntityGraph::incoming_node_indexes(graph, node_index);
            let weights = EntityGraph::incoming_weights(graph, node_index);
            let weight = weights[0] + weights[1];
            graph.add_edge(incoming[0], incoming[1], weight);
            graph.add_edge(incoming[1], incoming[0], weight);
            for connected_index in incoming {
                graph.remove_edge(graph.find_edge(connected_index, node_index).unwrap());
                graph.remove_edge(graph.find_edge(node_index, connected_index).unwrap());
            }
            graph.remove_node(node_index);
        }
    }
    pub fn remove(&self, entity: &FactorioEntity) -> anyhow::Result<()> {
        let mut inner = self.inner.write();
        let mut nodes_to_remove: Vec<NodeIndex> = vec![];
        let mut edges_to_remove: Vec<EdgeIndex> = vec![];

        if let Some(node_index) = EntityGraph::node_at(&inner, &entity.position) {
            for edge in inner.edges_directed(node_index, petgraph::Direction::Incoming) {
                edges_to_remove.push(edge.id());
            }
            for edge in inner.edges_directed(node_index, petgraph::Direction::Outgoing) {
                edges_to_remove.push(edge.id());
            }
            nodes_to_remove.push(node_index);
        }
        for edge in edges_to_remove {
            inner.remove_edge(edge);
        }
        for node in nodes_to_remove {
            inner.remove_node(node);
        }
        Ok(())
    }
    pub fn connect(&self) -> anyhow::Result<()> {
        let mut inner = self.inner.write();
        let mut edges_to_add: Vec<(NodeIndex, NodeIndex, f64)> = vec![];
        for node_index in inner.node_indices() {
            let node_index = node_index;
            let node = inner.node_weight(node_index).unwrap();
            if let Some(drop_position) = node.entity.drop_position.as_ref() {
                match EntityGraph::node_at(&inner, drop_position) {
                    Some(drop_index) => {
                        if !inner.contains_edge(node_index, drop_index) {
                            edges_to_add.push((node_index, drop_index, 1.));
                        }
                    }
                    None => error!(
                        "connect entity graph could not find entity at Drop position {} for {:?}",
                        drop_position, &node.entity
                    ),
                }
            }
            if let Some(pickup_position) = node.entity.pickup_position.as_ref() {
                match EntityGraph::node_at(&inner, pickup_position) {
                    Some(pickup_index) => {
                        if !inner.contains_edge(pickup_index, node_index) {
                            edges_to_add.push((pickup_index, node_index, 1.));
                        }
                    }
                    None => error!(
                        "connect entity graph could not find entity at Pickup position {} for {:?}",
                        pickup_position, &node.entity
                    ),
                }
            }
            match node.entity_type {
                EntityType::Splitter => {
                    let out1 = node
                        .entity
                        .position
                        .add(&Position::new(-0.5, -1.).turn(node.direction));
                    let out2 = node
                        .entity
                        .position
                        .add(&Position::new(0.5, -1.).turn(node.direction));
                    for pos in &[&out1, &out2] {
                        if let Some(next_index) = EntityGraph::node_at(&inner, pos) {
                            let next = inner.node_weight(next_index).unwrap();
                            // info!(
                            //     "found splitter output: {} @ {}",
                            //     next.entity.name, next.entity.position
                            // );
                            if !inner.contains_edge(node_index, next_index)
                                && self.is_entity_belt_connectable(node, next)
                            {
                                edges_to_add.push((node_index, next_index, 1.));
                            }
                            // } else {
                            //     warn!(
                            //         "NOT found splitter output: for {} @ {} -> searched @ {}",
                            //         node.entity.name, node.entity.position, pos
                            //     );
                        }
                    }
                }
                EntityType::TransportBelt => {
                    if let Some(next_index) = EntityGraph::node_at(
                        &inner,
                        &move_position(&node.entity.position, node.direction, 1.0),
                    ) {
                        let next = inner.node_weight(next_index).unwrap();
                        if !inner.contains_edge(node_index, next_index)
                            && self.is_entity_belt_connectable(node, next)
                        {
                            edges_to_add.push((node_index, next_index, 1.));
                        }
                        // } else {
                        //     warn!(
                        //         "not found transport belt connect from {} to {} ({:?})",
                        //         node.entity.position,
                        //         move_position(&node.entity.position, node.direction, 1.0),
                        //         node.direction
                        //     )
                    }
                }
                EntityType::OffshorePump => {
                    if let Some(next_index) = EntityGraph::node_at(
                        &inner,
                        &move_position(&node.entity.position, node.direction, -1.),
                    ) {
                        let next = inner.node_weight(next_index).unwrap();
                        if next.entity.is_fluid_input()
                            && !inner.contains_edge(node_index, next_index)
                        {
                            edges_to_add.push((node_index, next_index, 1.));
                        }
                    }
                }
                EntityType::Pipe => {
                    for direction in Direction::orthogonal() {
                        if let Some(next_index) = EntityGraph::node_at(
                            &inner,
                            &move_position(&node.entity.position, direction, 1.),
                        ) {
                            let next = inner.node_weight(next_index).unwrap();
                            if next.entity.is_fluid_input() {
                                if !inner.contains_edge(node_index, next_index) {
                                    edges_to_add.push((node_index, next_index, 1.));
                                }
                                if !inner.contains_edge(next_index, node_index) {
                                    edges_to_add.push((next_index, node_index, 1.));
                                }
                            }
                        }
                    }
                }
                EntityType::UndergroundBelt => {
                    let mut found = false;
                    for length in 1..5 {
                        // todo: lookup in EntityPrototypes for real belt length
                        if let Some(next_index) = EntityGraph::node_at(
                            &inner,
                            &move_position(
                                &node.entity.position,
                                node.direction.opposite(),
                                length as f64,
                            ),
                        ) {
                            let next = inner.node_weight(next_index).unwrap();
                            if next.entity_type == EntityType::UndergroundBelt {
                                if !inner.contains_edge(next_index, node_index) {
                                    edges_to_add.push((next_index, node_index, length as f64));
                                }
                                found = true;
                                break;
                            }
                        }
                    }
                    if found {
                        if let Some(next_index) = EntityGraph::node_at(
                            &inner,
                            &move_position(&node.entity.position, node.direction, 1.),
                        ) {
                            let next = inner.node_weight(next_index).unwrap();
                            if !inner.contains_edge(node_index, next_index)
                                && self.is_entity_belt_connectable(node, next)
                            {
                                edges_to_add.push((node_index, next_index, 1.));
                            }
                        }
                    }
                }
                EntityType::PipeToGround => {
                    let mut found = false;
                    for length in 1..12 {
                        // todo: lookup in EntityPrototypes for real pipe length
                        if let Some(next_index) = EntityGraph::node_at(
                            &inner,
                            &move_position(&node.entity.position, node.direction, -length as f64),
                        ) {
                            let next = inner.node_weight(next_index).unwrap();
                            if next.entity_type == EntityType::PipeToGround {
                                if !inner.contains_edge(next_index, node_index) {
                                    edges_to_add.push((next_index, node_index, length as f64));
                                }
                                if !inner.contains_edge(node_index, next_index) {
                                    edges_to_add.push((node_index, next_index, length as f64));
                                }
                                found = true;
                                break;
                            }
                        }
                    }
                    if found {
                        if let Some(next_index) = EntityGraph::node_at(
                            &inner,
                            &move_position(&node.entity.position, node.direction, 1.),
                        ) {
                            let next = inner.node_weight(next_index).unwrap();
                            if next.entity.is_fluid_input()
                                && !inner.contains_edge(node_index, next_index)
                            {
                                edges_to_add.push((node_index, next_index, 1.));
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        for (a, b, w) in edges_to_add {
            if !inner.contains_edge(a, b) {
                inner.add_edge(a, b, w);
            }
        }
        Ok(())
    }
    pub fn node_at(graph: &EntityGraphInner, position: &Position) -> Option<NodeIndex> {
        graph.node_indices().find(|i| {
            if let Some(f) = graph.node_weight(*i) {
                // info!(
                //     "checking for {} in {} {:?}",
                //     position, f.entity.name, f.entity.bounding_box
                // );
                return f.entity.bounding_box.contains(&position);
            }
            false
        })
    }
    fn is_entity_belt_connectable(&self, node: &EntityNode, next: &EntityNode) -> bool {
        (next.entity_type == EntityType::TransportBelt
            || next.entity_type == EntityType::UndergroundBelt
            || next.entity_type == EntityType::Splitter)
            && next.direction != node.direction.opposite()
    }
    pub fn graphviz_dot(&self) -> String {
        use petgraph::dot::{Config, Dot};
        format!(
            "digraph {{\n{:?}}}\n",
            Dot::with_config(&self.inner().deref(), &[Config::GraphContentOnly])
        )
    }
    pub fn graphviz_dot_condensed(&self) -> String {
        use petgraph::dot::{Config, Dot};
        let condensed = self.condense();
        format!(
            "digraph {{\n{:?}}}\n",
            Dot::with_config(&condensed, &[Config::GraphContentOnly])
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factorio::tests::{blueprint_entities, fixture_entity_prototypes};

    #[test]
    fn test_splitters() {
        let entities: Vec<FactorioEntity> = vec![
            FactorioEntity::new_transport_belt(&Position::new(0.5, 0.5), Direction::South),
            FactorioEntity::new_transport_belt(&Position::new(1.5, 0.5), Direction::South),
            FactorioEntity::new_splitter(&Position::new(1., 1.5), Direction::South),
            FactorioEntity::new_transport_belt(&Position::new(0.5, 2.5), Direction::South),
            FactorioEntity::new_transport_belt(&Position::new(1.5, 2.5), Direction::South),
        ];
        assert_eq!(
            EntityGraph::from(entities, |_name, _pos| false)
                .unwrap()
                .graphviz_dot(),
            r#"digraph {
    0 [ label = "transport-belt at [0.5, 0.5]" ]
    1 [ label = "transport-belt at [1.5, 0.5]" ]
    2 [ label = "splitter at [1, 1.5]" ]
    3 [ label = "transport-belt at [0.5, 2.5]" ]
    4 [ label = "transport-belt at [1.5, 2.5]" ]
    0 -> 2 [ label = "1.0" ]
    1 -> 2 [ label = "1.0" ]
    2 -> 4 [ label = "1.0" ]
    2 -> 3 [ label = "1.0" ]
}
"#,
        );
    }

    #[test]
    fn test_splitters2() {
        let (prototypes, _writer) = fixture_entity_prototypes();
        let entities: Vec<FactorioEntity> = blueprint_entities("0eNqd0u+KwyAMAPB3yWd3TK/q5quM42i3MITWimbHleK7n64clK1lf74ZMb8kkhGa9oI+WEdgRrDH3kUwhxGiPbu6LXc0eAQDlrADBq7uShR9a4kwQGJg3Ql/wfDEHqZRqF30faBNgy3NkkX6YoCOLFmcGrgGw7e7dE0uY/iawcD3Maf1rlTN1EbxD8lgAKPzIZc42YDH6YEoPd7I4n6oBXP7b4rH4uczotytiGpBrJ6fXu7n0y9Y8h1L3P5ktSCrF2S9KquyCte1MbPlZPCDIU5fvuOVrvZaab5VUqX0B2ef55s=", &prototypes).expect("failed to read blueprint");
        assert_eq!(
            EntityGraph::from(entities, |_name, _pos| false)
                .unwrap()
                .graphviz_dot(),
            r#"digraph {
    0 [ label = "transport-belt at [-61.5, 71.5]" ]
    1 [ label = "splitter at [-60.5, 72]" ]
    2 [ label = "splitter at [-58.5, 72]" ]
    3 [ label = "transport-belt at [-59.5, 71.5]" ]
    4 [ label = "transport-belt at [-59.5, 72.5]" ]
    5 [ label = "transport-belt at [-57.5, 72.5]" ]
    0 -> 1 [ label = "1.0" ]
    1 -> 3 [ label = "1.0" ]
    1 -> 4 [ label = "1.0" ]
    2 -> 4 [ label = "1.0" ]
    2 -> 3 [ label = "1.0" ]
    5 -> 2 [ label = "1.0" ]
}
"#,
        );
    }
}
