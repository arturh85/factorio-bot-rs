use crate::factorio::util::{add_to_rect, bounding_box, move_position, rect_fields, rect_floor};
use crate::num_traits::FromPrimitive;
use crate::types::{
    Direction, EntityName, EntityType, FactorioEntity, Pos, Position, Rect, ResourcePatch,
};
use aabb_quadtree::{ItemId, QuadTree};
use dashmap::lock::{RwLock, RwLockReadGuard};
use dashmap::DashMap;
use euclid::{TypedPoint2D, TypedRect, TypedSize2D};
use petgraph::graph::{EdgeIndex, NodeIndex};
use petgraph::stable_graph::StableGraph;
use petgraph::visit::{Bfs, EdgeRef};
use std::collections::HashMap;
use std::ops::Deref;
use std::str::FromStr;
use std::time::Instant;

#[derive(Default, Clone)]
pub struct EntityNode {
    pub bounding_box: Rect,
    pub position: Position,
    pub direction: Direction,
    pub entity_name: String,
    pub entity_type: EntityType,
    pub entity_id: Option<ItemId>,
    pub miner_ore: Option<String>,
}

impl std::fmt::Display for EntityNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "{}{} at {}",
            if let Some(miner_ore) = &self.miner_ore {
                format!("{}: ", miner_ore)
            } else {
                String::new()
            },
            self.entity_type,
            self.position
        ))?;
        Ok(())
    }
}
impl std::fmt::Debug for EntityNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "{}{} at {}",
            if let Some(miner_ore) = &self.miner_ore {
                format!("{}: ", miner_ore)
            } else {
                String::new()
            },
            self.entity_type,
            self.position
        ))?;
        Ok(())
    }
}

impl EntityNode {
    pub fn new(entity: FactorioEntity, miner_ore: Option<String>, entity_id: ItemId) -> EntityNode {
        let direction = Direction::from_u8(entity.direction).unwrap();
        let entity_type = EntityType::from_str(&entity.entity_type).unwrap();
        EntityNode {
            position: entity.position.clone(),
            bounding_box: entity.bounding_box.clone(),
            direction,
            miner_ore,
            entity_id: Some(entity_id),
            entity_name: entity.name,
            entity_type,
        }
    }
}

pub type EntityGraphInner = StableGraph<EntityNode, f64>;

pub type QuadTreeRect = TypedRect<f32, Rect>;
pub type EntityQuadTree = QuadTree<FactorioEntity, Rect, [(ItemId, QuadTreeRect); 4]>;

pub struct EntityGraph {
    entity_graph: RwLock<EntityGraphInner>,
    entity_tree: RwLock<EntityQuadTree>,
    entity_nodes: DashMap<ItemId, NodeIndex>,
    resources: DashMap<String, Vec<Pos>>,
}

impl EntityGraph {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        EntityGraph {
            entity_graph: RwLock::new(EntityGraphInner::new()),
            entity_tree: RwLock::new(QuadTree::new(
                QuadTreeRect::new(
                    TypedPoint2D::new(-5120., -5120.),
                    TypedSize2D::new(10240., 10240.),
                ),
                true,
                32,
                128,
                32,
                8,
            )),
            entity_nodes: DashMap::new(),
            resources: DashMap::new(),
        }
    }
    pub fn from(vec: Vec<FactorioEntity>) -> anyhow::Result<Self> {
        let graph = EntityGraph::new();
        graph.add(vec, None)?;
        graph.connect()?;
        Ok(graph)
    }
    pub fn inner_graph(&self) -> RwLockReadGuard<EntityGraphInner> {
        self.entity_graph.read()
    }
    pub fn inner_tree(&self) -> RwLockReadGuard<EntityQuadTree> {
        self.entity_tree.read()
    }

    fn walk(&self, m: &mut HashMap<Pos, Option<u32>>, pos: &Pos, id: u32) {
        m.insert(pos.clone(), Some(id));
        for direction in Direction::all() {
            let other: Pos = (&move_position(&pos.into(), direction, 1.0)).into();
            if let Some(p) = m.get(&other) {
                if p.is_none() {
                    self.walk(m, &other, id);
                }
            }
        }
    }
    pub fn resource_contains(&self, resource_name: &str, pos: Pos) -> bool {
        let elements = self.resources.get(resource_name);
        if let Some(elements) = elements {
            elements.contains(&pos)
        } else {
            false
        }
    }

    pub fn resource_patches(&self, resource_name: &str) -> Vec<ResourcePatch> {
        let mut patches: Vec<ResourcePatch> = vec![];
        let mut positions_by_id: HashMap<Pos, Option<u32>> = HashMap::new();
        for point in self
            .resources
            .get(resource_name)
            .expect("resource patch not found")
            .iter()
        {
            positions_by_id.insert(point.clone(), None);
        }
        let mut next_id: u32 = 0;

        while let Some((next_pos, _)) = positions_by_id.iter().find(|(_, value)| value.is_none()) {
            next_id += 1;
            let next_pos = next_pos.clone();
            self.walk(&mut positions_by_id, &next_pos, next_id);
        }
        for id in 1..=next_id {
            let mut elements: Vec<Position> = vec![];
            for (k, v) in &positions_by_id {
                if v.unwrap() == id {
                    elements.push(k.into());
                }
            }
            patches.push(ResourcePatch {
                name: resource_name.into(),
                rect: bounding_box(&elements).unwrap(),
                elements,
                id,
            });
        }
        patches.sort_by(|a, b| b.elements.len().cmp(&a.elements.len()));
        patches
    }

    pub fn add(
        &self,
        entities: Vec<FactorioEntity>,
        _clear_rect: Option<Rect>,
    ) -> anyhow::Result<()> {
        for entity in &entities {
            if entity.entity_type == EntityType::Resource.to_string() {
                match self.resources.get_mut(&entity.name) {
                    Some(mut positions) => {
                        positions.push((&entity.position).into());
                    }
                    None => {
                        self.resources
                            .insert(entity.name.clone(), vec![(&entity.position).into()]);
                    }
                }
            }
        }
        for entity in entities {
            if let Ok(entity_type) = EntityType::from_str(&entity.entity_type) {
                match entity_type {
                    EntityType::Furnace
                    | EntityType::Inserter
                    | EntityType::Boiler
                    | EntityType::Lab
                    | EntityType::OffshorePump
                    | EntityType::MiningDrill
                    | EntityType::Container
                    | EntityType::Splitter
                    | EntityType::TransportBelt
                    | EntityType::UndergroundBelt
                    | EntityType::Pipe
                    | EntityType::PipeToGround
                    | EntityType::LogisticContainer
                    | EntityType::AssemblingMachine => {
                        if let Some(entity_id) = self.entity_at(&entity.position) {
                            let tree = self.entity_tree.read();
                            let block = tree.get(entity_id).unwrap();
                            warn!(
                                "failed to add {}@{} -> blocked by {}@{}",
                                entity.name, entity.position, block.name, block.position
                            );
                            continue;
                        }
                        if let Some(entity_id) = {
                            let mut tree = self.entity_tree.write();
                            tree.insert(entity.clone())
                        } {
                            let miner_ore = if entity_type == EntityType::MiningDrill {
                                let rect = rect_floor(&entity.bounding_box);
                                let mut miner_ore: Option<String> = None;
                                for resource in &[
                                    EntityName::IronOre,
                                    EntityName::CopperOre,
                                    EntityName::Coal,
                                    EntityName::Stone,
                                    EntityName::CrudeOil,
                                    EntityName::UraniumOre,
                                ] {
                                    let resource = resource.to_string();
                                    let resource_found = rect_fields(&rect).iter().any(|p| {
                                        self.resources
                                            .get(&resource)
                                            .and_then(|resources| {
                                                if resources.contains(&p.into()) {
                                                    Some(true)
                                                } else {
                                                    None
                                                }
                                            })
                                            .is_some()
                                    });
                                    if resource_found {
                                        miner_ore = Some(resource);
                                        break;
                                    }
                                }
                                if miner_ore.is_none() {
                                    warn!(
                                        "no ore found under miner {} @ {}",
                                        entity.name, entity.position
                                    );
                                }
                                miner_ore
                            } else {
                                None
                            };
                            let new_node = EntityNode::new(entity.clone(), miner_ore, entity_id);
                            let mut inner = self.entity_graph.write();
                            let new_node_index = inner.add_node(new_node);
                            self.entity_nodes.insert(entity_id, new_node_index);
                        } else {
                            warn!("failed to insert entity into quad tree");
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    pub fn condense(&self) -> EntityGraphInner {
        let _started = Instant::now();
        let mut graph = self.entity_graph.read().clone();
        let _starting_nodes = graph.node_indices().count();
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
                self.condense_walk(&mut graph, next_node);
            } else {
                break;
            }
        }

        let mut orphans: Vec<NodeIndex> = vec![];
        for node_index in graph.node_indices() {
            if graph
                .edges_directed(node_index, petgraph::Direction::Incoming)
                .count()
                == 0
                && graph
                    .edges_directed(node_index, petgraph::Direction::Outgoing)
                    .count()
                    == 0
            {
                orphans.push(node_index);
            }
        }
        for orphan in orphans {
            graph.remove_node(orphan);
        }
        // info!(
        //     "condensing entity graph from {} to {} entities took {:?}",
        //     starting_nodes,
        //     graph.node_indices().count(),
        //     started.elapsed()
        // );
        graph
    }
    pub fn condense_walk(&self, graph: &mut EntityGraphInner, node_index: NodeIndex) {
        let mut bfs = Bfs::new(&*graph, node_index);
        while let Some(node_index) = bfs.next(&*graph) {
            let node = graph.node_weight(node_index).unwrap();
            let incoming: Vec<String> = graph
                .edges_directed(node_index, petgraph::Direction::Incoming)
                .map(|edge| {
                    graph
                        .node_weight(edge.target())
                        .unwrap()
                        .entity_name
                        .clone()
                })
                .collect();
            let outgoing: Vec<String> = graph
                .edges_directed(node_index, petgraph::Direction::Outgoing)
                .map(|edge| {
                    graph
                        .node_weight(edge.target())
                        .unwrap()
                        .entity_name
                        .clone()
                })
                .collect();
            if incoming.len() == 1
                && outgoing.len() == 1
                && node.entity_name == incoming[0]
                && incoming[0] == outgoing[0]
            {
                let incoming: NodeIndex = graph
                    .edges_directed(node_index, petgraph::Direction::Incoming)
                    .map(|edge| edge.source())
                    .find(|_| true)
                    .unwrap();
                let outgoing = graph
                    .edges_directed(node_index, petgraph::Direction::Outgoing)
                    .map(|edge| edge.target())
                    .find(|_| true)
                    .unwrap();
                let weight = graph
                    .edges_directed(node_index, petgraph::Direction::Incoming)
                    .map(|edge| *edge.weight())
                    .find(|_| true)
                    .unwrap()
                    + graph
                        .edges_directed(node_index, petgraph::Direction::Outgoing)
                        .map(|edge| *edge.weight())
                        .find(|_| true)
                        .unwrap();
                self.entity_nodes.remove(&node.entity_id.unwrap());
                graph.add_edge(incoming, outgoing, weight);
                if let Some(edge) = graph.find_edge(incoming, node_index) {
                    graph.remove_edge(edge);
                }
                if let Some(edge) = graph.find_edge(node_index, outgoing) {
                    graph.remove_edge(edge);
                }
                graph.remove_node(node_index);
            } else if incoming.len() == 2
                && outgoing.len() == 2
                && node.entity_name == incoming[0]
                && incoming[0] == outgoing[0]
            {
                let incoming: Vec<NodeIndex> = graph
                    .edges_directed(node_index, petgraph::Direction::Incoming)
                    .map(|edge| edge.source())
                    .collect();
                let weights: Vec<f64> = graph
                    .edges_directed(node_index, petgraph::Direction::Incoming)
                    .map(|edge| *edge.weight())
                    .collect();
                let weight = weights[0] + weights[1];
                self.entity_nodes.remove(&node.entity_id.unwrap());
                graph.add_edge(incoming[0], incoming[1], weight);
                graph.add_edge(incoming[1], incoming[0], weight);
                for connected_index in incoming {
                    if let Some(edge) = graph.find_edge(connected_index, node_index) {
                        graph.remove_edge(edge);
                    }
                    if let Some(edge) = graph.find_edge(node_index, connected_index) {
                        graph.remove_edge(edge);
                    }
                }
                graph.remove_node(node_index);
            }
        }
    }
    pub fn remove(&self, entity: &FactorioEntity) -> anyhow::Result<()> {
        let mut nodes_to_remove: Vec<NodeIndex> = vec![];
        let mut edges_to_remove: Vec<EdgeIndex> = vec![];
        let mut entities_to_remove: Vec<ItemId> = vec![];

        if let Some(entity_id) = self.entity_at(&entity.position) {
            if let Some(node_index) = self.entity_nodes.get(&entity_id) {
                let inner = self.entity_graph.read();
                for edge in inner.edges_directed(*node_index, petgraph::Direction::Incoming) {
                    edges_to_remove.push(edge.id());
                }
                for edge in inner.edges_directed(*node_index, petgraph::Direction::Outgoing) {
                    edges_to_remove.push(edge.id());
                }
                nodes_to_remove.push(*node_index);
            }
            entities_to_remove.push(entity_id);
        }
        let mut inner = self.entity_graph.write();
        for edge in edges_to_remove {
            inner.remove_edge(edge);
        }
        for entity_id in entities_to_remove {
            self.entity_nodes.remove(&entity_id);
        }
        for node in nodes_to_remove {
            inner.remove_node(node);
        }
        Ok(())
    }

    pub fn connect(&self) -> anyhow::Result<()> {
        let _started = Instant::now();
        let tree = self.entity_tree.read();
        let mut edges_to_add: Vec<(NodeIndex, NodeIndex, f64)> = vec![];
        let nodes: Vec<NodeIndex> = self.entity_graph.read().node_indices().map(|i| i).collect();
        for node_index in nodes {
            let inner = self.entity_graph.read();
            let node_index = node_index;
            if let Some(node) = inner.node_weight(node_index) {
                let node_entity = tree.get(node.entity_id.unwrap()).unwrap();
                if let Some(drop_position) = node_entity.drop_position.as_ref() {
                    match self.node_at(drop_position) {
                        Some(drop_index) => {
                            if !inner.contains_edge(node_index, drop_index) {
                                edges_to_add.push((node_index, drop_index, 1.));
                            }
                        }
                        None => error!(
                            "connect entity graph could not find entity at Drop position {} for {} @ {}",
                            drop_position, node_entity.name, node_entity.position
                        ),
                    }
                }
                if let Some(pickup_position) = node_entity.pickup_position.as_ref() {
                    match self.node_at(pickup_position) {
                    Some(pickup_index) => {
                        if !inner.contains_edge(pickup_index, node_index) {
                            edges_to_add.push((pickup_index, node_index, 1.));
                        }
                    }
                    None => error!(
                        "connect entity graph could not find entity at Pickup position {} for {} @ {}",
                        pickup_position, node_entity.name, node_entity.position
                    ),
                }
                }
                match node.entity_type {
                    EntityType::Splitter => {
                        let out1 = node
                            .position
                            .add(&Position::new(-0.5, -1.).turn(node.direction));
                        let out2 = node
                            .position
                            .add(&Position::new(0.5, -1.).turn(node.direction));
                        for pos in &[&out1, &out2] {
                            if let Some(next_index) = self.node_at(pos) {
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
                        if let Some(next_index) =
                            self.node_at(&move_position(&node.position, node.direction, 1.0))
                        {
                            let next = inner.node_weight(next_index).unwrap();
                            if !inner.contains_edge(node_index, next_index)
                                && self.is_entity_belt_connectable(node, next)
                            {
                                edges_to_add.push((node_index, next_index, 1.));
                                // } else {
                                //     warn!(
                                //         "2 not found transport belt connect from {} to {} ({:?})",
                                //         node.position,
                                //         move_position(&node.position, node.direction, 1.0),
                                //         node.direction
                                //     )
                            }
                            // } else {
                            //     warn!(
                            //         "1 not found transport belt connect from {} to {} ({:?})",
                            //         node.position,
                            //         move_position(&node.position, node.direction, 1.0),
                            //         node.direction
                            //     )
                        }
                    }
                    EntityType::OffshorePump => {
                        if let Some(next_index) =
                            self.node_at(&move_position(&node.position, node.direction, -1.))
                        {
                            let next = inner.node_weight(next_index).unwrap();
                            if next.entity_type.is_fluid_input()
                                && !inner.contains_edge(node_index, next_index)
                            {
                                edges_to_add.push((node_index, next_index, 1.));
                            }
                        }
                    }
                    EntityType::Pipe => {
                        for direction in Direction::orthogonal() {
                            if let Some(next_index) =
                                self.node_at(&move_position(&node.position, direction, 1.))
                            {
                                let next = inner.node_weight(next_index).unwrap();
                                if next.entity_type.is_fluid_input() {
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
                            if let Some(next_index) = self.node_at(&move_position(
                                &node.position,
                                node.direction.opposite(),
                                length as f64,
                            )) {
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
                            if let Some(next_index) =
                                self.node_at(&move_position(&node.position, node.direction, 1.))
                            {
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
                            if let Some(next_index) = self.node_at(&move_position(
                                &node.position,
                                node.direction,
                                -length as f64,
                            )) {
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
                            if let Some(next_index) =
                                self.node_at(&move_position(&node.position, node.direction, 1.))
                            {
                                let next = inner.node_weight(next_index).unwrap();
                                if next.entity_type.is_fluid_input()
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
        }
        let mut inner = self.entity_graph.write();
        for (a, b, w) in edges_to_add {
            if !inner.contains_edge(a, b) {
                inner.add_edge(a, b, w);
            }
        }
        // info!(
        //     "entity graph connecting {} entities took {:?}",
        //     inner.node_indices().count(),
        //     started.elapsed()
        // );
        Ok(())
    }
    pub fn entity_by_id(&self, id: ItemId) -> Option<FactorioEntity> {
        self.entity_tree.read().get(id).cloned()
    }

    pub fn node_at(&self, position: &Position) -> Option<NodeIndex> {
        self.entity_at(position)
            .and_then(|entity_id| self.entity_nodes.get(&entity_id).map(|e| *e))
    }

    pub fn entity_at(&self, position: &Position) -> Option<ItemId> {
        let tree = self.entity_tree.read();
        let results: Vec<ItemId> = tree
            .query(add_to_rect(&Rect::from_wh(0.1, 0.1), position).into())
            .iter()
            .map(|(_entity, _rect, item_id)| *item_id)
            .collect();

        if results.is_empty() {
            None
        } else if results.len() == 1 {
            Some(results[0])
        } else {
            panic!("multiple entity quad tree results for {}", position);
        }
    }
    pub fn entity_at_aabb(&self, rect: &Rect) -> Option<ItemId> {
        let tree = self.entity_tree.read();
        let rect: QuadTreeRect = rect.clone().into();
        let results: Vec<ItemId> = tree
            .query(rect)
            .iter()
            .map(|(_entity, _rect, item_id)| *item_id)
            .collect();
        if results.is_empty() {
            None
        } else if results.len() == 1 {
            Some(results[0])
        } else {
            panic!("multiple entity quad tree results for {:?}", rect);
        }
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
            Dot::with_config(&self.inner_graph().deref(), &[Config::GraphContentOnly])
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

    pub fn node_weight(&self, i: NodeIndex) -> Option<EntityNode> {
        self.entity_graph.read().node_weight(i).cloned()
    }
    pub fn edges_directed(&self, i: NodeIndex, dir: petgraph::Direction) -> Vec<NodeIndex> {
        self.entity_graph
            .read()
            .edges_directed(i, dir)
            .map(|e| e.target())
            .collect()
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
            EntityGraph::from(entities).unwrap().graphviz_dot(),
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
    fn test_condense() {
        let entities: Vec<FactorioEntity> = vec![
            FactorioEntity::new_transport_belt(&Position::new(0.5, 0.5), Direction::South),
            FactorioEntity::new_transport_belt(&Position::new(0.5, 1.5), Direction::South),
            FactorioEntity::new_transport_belt(&Position::new(0.5, 2.5), Direction::South),
            FactorioEntity::new_transport_belt(&Position::new(0.5, 3.5), Direction::South),
            FactorioEntity::new_transport_belt(&Position::new(0.5, 4.5), Direction::South),
        ];
        assert_eq!(
            EntityGraph::from(entities)
                .unwrap()
                .graphviz_dot_condensed(),
            r#"digraph {
    0 [ label = "transport-belt at [0.5, 0.5]" ]
    4 [ label = "transport-belt at [0.5, 4.5]" ]
    0 -> 4 [ label = "4.0" ]
}
"#,
        );
    }

    #[test]
    fn test_splitters2() {
        let (prototypes, _writer) = fixture_entity_prototypes();
        let entities: Vec<FactorioEntity> = blueprint_entities("0eNqd0u+KwyAMAPB3yWd3TK/q5quM42i3MITWimbHleK7n64clK1lf74ZMb8kkhGa9oI+WEdgRrDH3kUwhxGiPbu6LXc0eAQDlrADBq7uShR9a4kwQGJg3Ql/wfDEHqZRqF30faBNgy3NkkX6YoCOLFmcGrgGw7e7dE0uY/iawcD3Maf1rlTN1EbxD8lgAKPzIZc42YDH6YEoPd7I4n6oBXP7b4rH4uczotytiGpBrJ6fXu7n0y9Y8h1L3P5ktSCrF2S9KquyCte1MbPlZPCDIU5fvuOVrvZaab5VUqX0B2ef55s=", &prototypes).expect("failed to read blueprint");
        assert_eq!(
            EntityGraph::from(entities).unwrap().graphviz_dot(),
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
