use crate::factorio::entity_graph::{EntityGraphInner, EntityNode};
use crate::num_traits::FromPrimitive;
use crate::types::{Direction, EntityType, FactorioEntity, FactorioEntityPrototype, Position};
use evmap::ReadHandle;
use num_traits::ToPrimitive;
use petgraph::graph::NodeIndex;
use petgraph::stable_graph::StableGraph;
use petgraph::visit::EdgeRef;
use std::collections::HashMap;
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

#[derive(Clone, Debug)]
pub enum FlowEdge {
    Single(Vec<(String, f64)>),
    Double(Vec<(String, f64)>, Vec<(String, f64)>),
}

impl FlowEdge {
    pub fn split(&self) -> FlowEdge {
        match self {
            FlowEdge::Single(vec) => FlowEdge::Single(
                vec.iter()
                    .map(|(name, production_rate)| (name.clone(), production_rate / 2.))
                    .collect(),
            ),
            FlowEdge::Double(left, right) => FlowEdge::Double(
                left.iter()
                    .map(|(name, production_rate)| (name.clone(), production_rate / 2.))
                    .collect(),
                right
                    .iter()
                    .map(|(name, production_rate)| (name.clone(), production_rate / 2.))
                    .collect(),
            ),
        }
    }
}

impl Default for FlowEdge {
    fn default() -> Self {
        FlowEdge::Single(vec![])
    }
}

pub type FlowGraphInner = StableGraph<FlowNode, FlowEdge>;
pub type FlowRates = Vec<(String, f64)>;

pub struct FlowGraph {
    inner: FlowGraphInner,
}

impl FlowGraph {
    pub fn new(
        entity_prototypes: &ReadHandle<String, FactorioEntityPrototype>,
        entity_graph: &EntityGraphInner,
    ) -> Self {
        let mut graph = FlowGraphInner::new();
        let mut i = 0;
        for node_index in entity_graph.node_indices() {
            let node = entity_graph.node_weight(node_index).unwrap();
            if node.entity_type == EntityType::MiningDrill && node.miner_ore.is_some() {
                let root =
                    graph.add_node(FlowNode::new(node.entity.clone(), node.miner_ore.clone()));
                FlowGraph::walk(
                    entity_prototypes,
                    entity_graph,
                    &mut graph,
                    node_index,
                    root,
                );
                i += 1;
                if i > 3 {
                    // break;
                }
            }
        }
        FlowGraph { inner: graph }
    }

    pub fn node_at(graph: &FlowGraphInner, position: &Position) -> Option<NodeIndex> {
        graph.node_indices().find(|i| {
            if let Some(f) = graph.node_weight(*i) {
                return f.entity.bounding_box.contains(&position);
            }
            false
        })
    }

    fn walk(
        entity_prototypes: &ReadHandle<String, FactorioEntityPrototype>,
        entity_graph: &EntityGraphInner,
        graph: &mut FlowGraphInner,
        entity_node_index: NodeIndex,
        flow_node_index: NodeIndex,
    ) {
        let entity_node = entity_graph.node_weight(entity_node_index).unwrap();
        match entity_node.entity_type {
            EntityType::MiningDrill => {
                // should have no incoming and exactly one outgoing
                if let Some((next_entity_node_index, next_entity_node)) =
                    FlowGraph::outgoing_entities(entity_graph, entity_node_index).pop()
                {
                    let next_flow_node_index =
                        FlowGraph::outgoing_flow(graph, &next_entity_node, flow_node_index);
                    let miner_ore = entity_node.miner_ore.as_ref().unwrap();
                    let mining_speed = entity_prototypes
                        .get_one(&entity_node.entity.name)
                        .unwrap()
                        .mining_speed
                        .unwrap()
                        .to_f64()
                        .unwrap();
                    let mining_time = entity_prototypes
                        .get_one(miner_ore)
                        .unwrap()
                        .mining_time
                        .unwrap()
                        .to_f64()
                        .unwrap();
                    // https://wiki.factorio.com/Mining
                    // The rate at which resources are produced is given by:
                    // Mining speed / Mining time = Production rate (in resource/sec)
                    let production_rate = mining_speed / mining_time;
                    graph.add_edge(
                        flow_node_index,
                        next_flow_node_index,
                        FlowEdge::Single(vec![(miner_ore.clone(), production_rate)]),
                    );
                    FlowGraph::walk(
                        entity_prototypes,
                        entity_graph,
                        graph,
                        next_entity_node_index,
                        next_flow_node_index,
                    );
                }
            }
            EntityType::Assembler => {
                // can have multiple incoming and multiple outgoing
            }
            EntityType::Splitter => {
                // can have multiple incoming and multiple outgoing
                let incoming = FlowGraph::sum_incoming_edge_weights(graph, flow_node_index);
                let outgoing = FlowGraph::outgoing_entities(entity_graph, entity_node_index);
                for (next_entity_node_index, next_entity_node) in outgoing {
                    let next_flow_node_index =
                        FlowGraph::outgoing_flow(graph, &next_entity_node, flow_node_index);
                    graph.update_edge(
                        flow_node_index,
                        next_flow_node_index,
                        FlowEdge::Single(incoming.clone()).split(),
                    );
                    FlowGraph::walk(
                        entity_prototypes,
                        entity_graph,
                        graph,
                        next_entity_node_index,
                        next_flow_node_index,
                    );
                }
            }
            EntityType::Furnace => {
                // can have multiple incoming and multiple outgoing
                // let _incoming = sum_incoming_edge_weights(graph, flow_node_index);
                /*
                Smelting iron, copper, and stone each take a base 3.2 seconds to finish.
                Smelting steel takes a base 16 seconds.
                Stone Furnaces have a crafting speed of 1.
                Both Steel and Electric Furnaces have a crafting speed of 2.
                One furnace making iron can support one furnace making steel.
                Stone and Steel Furnaces consume 0.0225 coal/second.
                             */
            }
            EntityType::TransportBelt | EntityType::UndergroundBelt => {
                // can have multiple incoming and multiple outgoing
                let (left, right) = FlowGraph::sum_incoming_edge_weights_by_side(
                    graph,
                    entity_graph,
                    entity_node_index,
                    flow_node_index,
                );
                for (next_entity_node_index, next_entity_node) in
                    FlowGraph::outgoing_entities(entity_graph, entity_node_index)
                {
                    match next_entity_node.entity_type {
                        EntityType::TransportBelt
                        | EntityType::UndergroundBelt
                        | EntityType::Splitter => {
                            let next_flow_node_index =
                                FlowGraph::outgoing_flow(graph, &next_entity_node, flow_node_index);
                            graph.update_edge(
                                flow_node_index,
                                next_flow_node_index,
                                FlowEdge::Double(left.clone(), right.clone()),
                            );
                            FlowGraph::walk(
                                entity_prototypes,
                                entity_graph,
                                graph,
                                next_entity_node_index,
                                next_flow_node_index,
                            );
                        }
                        EntityType::Inserter => {
                            let next_flow_node_index =
                                FlowGraph::outgoing_flow(graph, &next_entity_node, flow_node_index);

                            let mut both = left.clone();
                            for e in &right {
                                FlowGraph::add_production_rate(&mut both, e.clone());
                            }
                            graph.update_edge(
                                flow_node_index,
                                next_flow_node_index,
                                FlowEdge::Single(both.clone()),
                            );
                            FlowGraph::walk(
                                entity_prototypes,
                                entity_graph,
                                graph,
                                next_entity_node_index,
                                next_flow_node_index,
                            );
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    fn outgoing_entities(
        graph: &EntityGraphInner,
        entity_node_index: NodeIndex,
    ) -> Vec<(NodeIndex, &EntityNode)> {
        graph
            .edges_directed(entity_node_index, petgraph::Direction::Outgoing)
            .into_iter()
            .map(|edge| (edge.target(), graph.node_weight(edge.target()).unwrap()))
            .collect()
    }

    fn outgoing_flow(
        graph: &mut FlowGraphInner,
        entity_node: &EntityNode,
        flow_node_index: NodeIndex,
    ) -> NodeIndex {
        let existing: Vec<NodeIndex> = graph
            .edges_directed(flow_node_index, petgraph::Direction::Outgoing)
            .into_iter()
            .map(|edge| edge.target())
            .collect();
        match existing.is_empty() {
            true => match FlowGraph::node_at(graph, &entity_node.entity.position) {
                Some(node_index) => node_index,
                None => graph.add_node(FlowNode::new(
                    entity_node.entity.clone(),
                    entity_node.miner_ore.clone(),
                )),
            },
            false => *existing.first().unwrap(),
        }
    }

    fn sum_incoming_edge_weights_by_side(
        graph: &FlowGraphInner,
        entity_graph: &EntityGraphInner,
        entity_node_index: NodeIndex,
        flow_node_index: NodeIndex,
    ) -> (FlowRates, FlowRates) {
        let mut left: FlowRates = vec![];
        let mut right: FlowRates = vec![];

        let flow_node = graph.node_weight(flow_node_index).unwrap();

        for edge in graph.edges_directed(flow_node_index, petgraph::Direction::Incoming) {
            let weight = edge.weight();
            let prev_node = graph.node_weight(edge.source()).unwrap();
            let entity_edge_count = entity_graph
                .edges_directed(entity_node_index, petgraph::Direction::Incoming)
                // .filter(|edge| {
                //     let entity_type = &entity_graph.node_weight(edge.source()).unwrap().entity_type;
                //     entity_type == &EntityType::TransportBelt
                //         || entity_type == &EntityType::UndergroundBelt
                // })
                .count();

            info!(
                "sum by -> flow {} @ {} {:?} next {} @ {} {:?} -> edges {}",
                flow_node.entity.name,
                flow_node.entity.position,
                flow_node.direction,
                prev_node.entity.name,
                prev_node.entity.position,
                prev_node.direction,
                entity_edge_count
            );
            // let intersection_left = flow_node.direction.clockwise() == prev_node.direction && entity_node_at(entity_graph, Position::)

            if flow_node.direction == prev_node.direction || entity_edge_count == 1 {
                match weight {
                    FlowEdge::Single(vec) => {
                        for (name, production_rate) in vec {
                            FlowGraph::add_production_rate(
                                &mut left,
                                (name.clone(), production_rate / 2.),
                            );
                            FlowGraph::add_production_rate(
                                &mut right,
                                (name.clone(), production_rate / 2.),
                            );
                        }
                    }
                    FlowEdge::Double(l, r) => {
                        for e in l {
                            FlowGraph::add_production_rate(&mut left, e.clone());
                        }
                        for e in r {
                            FlowGraph::add_production_rate(&mut right, e.clone());
                        }
                    }
                }
            } else if flow_node.direction.clockwise().opposite() == prev_node.direction {
                match weight {
                    FlowEdge::Single(vec) => {
                        for (name, production_rate) in vec {
                            FlowGraph::add_production_rate(
                                &mut right,
                                (name.clone(), *production_rate),
                            );
                        }
                    }
                    FlowEdge::Double(l, r) => {
                        for e in l {
                            FlowGraph::add_production_rate(&mut right, e.clone());
                        }
                        for e in r {
                            FlowGraph::add_production_rate(&mut right, e.clone());
                        }
                    }
                }
            } else if flow_node.direction.clockwise() == prev_node.direction {
                match weight {
                    FlowEdge::Single(vec) => {
                        for (name, production_rate) in vec {
                            FlowGraph::add_production_rate(
                                &mut left,
                                (name.clone(), *production_rate),
                            );
                        }
                    }
                    FlowEdge::Double(l, r) => {
                        for e in l {
                            FlowGraph::add_production_rate(&mut left, e.clone());
                        }
                        for e in r {
                            FlowGraph::add_production_rate(&mut left, e.clone());
                        }
                    }
                }
            }
        }

        (left, right)
    }

    fn add_production_rate(vec: &mut FlowRates, entry: (String, f64)) {
        match vec.iter_mut().find(|e| e.0 == entry.0) {
            Some(e) => e.1 += entry.1,
            None => vec.push(entry),
        }
    }

    fn sum_production_rates(input: Vec<FlowRates>) -> FlowRates {
        let mut map: HashMap<String, f64> = HashMap::new();
        for vec in input {
            for (name, production_rate) in vec {
                if let Some(v) = map.get(&name) {
                    let v = *v;
                    map.insert(name, v + production_rate);
                } else {
                    map.insert(name, production_rate);
                }
            }
        }
        map.into_iter()
            .map(|(name, production_rate)| (name, production_rate))
            .collect()
    }

    fn sum_incoming_edge_weights(graph: &FlowGraphInner, flow_node_index: NodeIndex) -> FlowRates {
        let incoming: Vec<FlowEdge> = graph
            .edges_directed(flow_node_index, petgraph::Direction::Incoming)
            .map(|i| graph.edge_weight(i.id()).unwrap().clone())
            .collect();
        let mut rates: Vec<FlowRates> = vec![];
        for edge in incoming {
            match edge {
                FlowEdge::Single(vec) => {
                    rates.push(vec);
                }
                FlowEdge::Double(left, right) => {
                    rates.push(left);
                    rates.push(right);
                }
            }
        }
        FlowGraph::sum_production_rates(rates)
    }
    pub fn graphviz_dot(&self) -> String {
        use petgraph::dot::{Config, Dot};
        format!(
            "digraph {{\n{:?}}}\n",
            Dot::with_config(&self.inner, &[Config::GraphContentOnly])
        )
    }
}
