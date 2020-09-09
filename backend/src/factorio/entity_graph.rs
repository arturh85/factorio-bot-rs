use crate::factorio::util::{move_position, rect_fields, rect_floor};
use crate::factorio::world::FactorioWorld;
use crate::num_traits::FromPrimitive;
use crate::types::{Direction, EntityName, EntityType, FactorioEntity, Position};
use atomic_refcell::{AtomicRef, AtomicRefCell};
use petgraph::graph::{EdgeIndex, NodeIndex};
use petgraph::stable_graph::StableGraph;
use petgraph::visit::EdgeRef;
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
    inner: AtomicRefCell<EntityGraphInner>,
    // quad_tree: Quadtree<i32, NodeIndex>,
}

impl EntityGraph {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        EntityGraph {
            inner: AtomicRefCell::new(EntityGraphInner::new()),
            // quad_tree: Quadtree::new(16),
        }
    }
    pub fn inner(&self) -> AtomicRef<EntityGraphInner> {
        self.inner.borrow()
    }
    pub fn add(&self, entity: &FactorioEntity, world: &FactorioWorld) -> anyhow::Result<()> {
        let mut inner = self.inner.borrow_mut();
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
                                    .any(|p| world.resource_contains(&resource, p.into()));
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
    pub fn remove(&self, entity: &FactorioEntity) -> anyhow::Result<()> {
        let mut inner = self.inner.borrow_mut();
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
        let mut inner = self.inner.borrow_mut();
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
                    let in1 = node
                        .entity
                        .position
                        .add(&Position::new(-0.5, 1.).turn(node.direction));
                    let in2 = node
                        .entity
                        .position
                        .add(&Position::new(0.5, 1.).turn(node.direction));
                    for pos in vec![in1, in2] {
                        if let Some(prev_index) = EntityGraph::node_at(&inner, &pos) {
                            let prev = inner.node_weight(prev_index).unwrap();
                            info!("splitter inputs {}", pos);
                            if !inner.contains_edge(node_index, prev_index)
                                && self.is_entity_belt_connectable(node, prev)
                            {
                                edges_to_add.push((node_index, prev_index, 1.));
                            }
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
}
