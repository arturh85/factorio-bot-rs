use crate::num_traits::FromPrimitive;
use crate::types::{Direction, FactorioEntity, Position};
use noisy_float::types::{r64, R64};
use petgraph::algo::astar;
use petgraph::graph::{DefaultIx, EdgeIndex, NodeIndex};
use petgraph::stable_graph::{Edges, NodeIndices, StableGraph};
use petgraph::Directed;
use serde::export::Formatter;

#[derive(Debug, Clone)]
pub enum TaskData {
    Mine(MineTarget),
    Walk(PositionRadius),
    Craft(InventoryItem),
    InsertToInventory(InventoryLocation, InventoryItem),
    RemoveFromInventory(InventoryLocation, InventoryItem),
    PlaceEntity(FactorioEntity),
}

#[derive(Default, Clone)]
pub struct TaskNode {
    pub name: String,
    pub player_id: Option<u32>,
    pub data: Option<TaskData>,
}

impl TaskNode {
    pub fn new(player_id: Option<u32>, name: &str, data: Option<TaskData>) -> TaskNode {
        TaskNode {
            name: name.into(),
            player_id,
            data,
        }
    }
    pub fn new_craft(player_id: u32, item: InventoryItem) -> TaskNode {
        TaskNode::new(
            Some(player_id),
            &*format!(
                "Craft {}{}",
                item.name,
                if item.count > 1 {
                    format!(" x {}", item.count)
                } else {
                    String::new()
                }
            ),
            Some(TaskData::Craft(item)),
        )
    }
    pub fn new_walk(player_id: u32, target: PositionRadius) -> TaskNode {
        TaskNode::new(
            Some(player_id),
            &*format!("Walk to {}", target.position),
            Some(TaskData::Walk(target)),
        )
    }
    pub fn new_mine(player_id: u32, target: MineTarget) -> TaskNode {
        TaskNode::new(
            Some(player_id),
            &*format!(
                "Mining {}{}",
                target.name,
                if target.count > 1 {
                    format!(" x {}", target.count)
                } else {
                    String::new()
                }
            ),
            Some(TaskData::Mine(target)),
        )
    }
    pub fn new_place(player_id: u32, entity: FactorioEntity) -> TaskNode {
        TaskNode::new(
            Some(player_id),
            &*format!(
                "Place {} at {} ({:?})",
                entity.name,
                entity.position,
                Direction::from_u8(entity.direction).unwrap()
            ),
            Some(TaskData::PlaceEntity(entity)),
        )
    }
    pub fn new_insert_to_inventory(
        player_id: u32,
        location: InventoryLocation,
        item: InventoryItem,
    ) -> TaskNode {
        TaskNode::new(
            Some(player_id),
            &*format!(
                "Insert {}x{} into {} at {}",
                &item.name, &item.count, location.entity_name, location.position
            ),
            Some(TaskData::InsertToInventory(location, item)),
        )
    }
}

impl std::fmt::Display for TaskNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name)?;
        Ok(())
    }
}
impl std::fmt::Debug for TaskNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name)?;
        Ok(())
    }
}

pub struct TaskResult(i32);

pub type TaskGraphInner = StableGraph<TaskNode, f64>;

#[derive(Clone)]
pub struct TaskGraph {
    inner: TaskGraphInner,
}

impl TaskGraph {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        TaskGraph {
            inner: TaskGraphInner::new(),
        }
    }
    pub fn weight(&self, start: NodeIndex, goal: NodeIndex) -> R64 {
        let (weight, _) = self.astar(start, goal).expect("failed to find path");
        r64(weight)
    }

    pub fn node_indices(&self) -> NodeIndices<TaskNode, DefaultIx> {
        self.inner.node_indices()
    }

    pub fn add_process_start_node(&mut self, parent: NodeIndex, label: &str) -> NodeIndex {
        let start = self
            .inner
            .add_node(TaskNode::new(None, &format!("Start: {}", label), None));
        self.inner.add_edge(parent, start, 0.);

        start
    }
    pub fn add_process_end_node(&mut self) -> NodeIndex {
        self.inner.add_node(TaskNode::new(None, "End", None))
    }
    pub fn graphviz_dot(&self) -> String {
        use petgraph::dot::{Config, Dot};
        format!(
            "digraph {{\n{:?}}}\n",
            Dot::with_config(&self.inner, &[Config::GraphContentOnly])
        )
    }

    pub fn add_node(&mut self, task: TaskNode) -> NodeIndex {
        self.inner.add_node(task)
    }

    pub fn add_edge(&mut self, a: NodeIndex, b: NodeIndex, weight: f64) -> EdgeIndex {
        self.inner.add_edge(a, b, weight)
    }

    pub fn astar(&self, start: NodeIndex, goal: NodeIndex) -> Option<(f64, Vec<NodeIndex>)> {
        astar(
            &self.inner,
            start,
            |finish| finish == goal,
            |e| *e.weight(),
            |_| 0.,
        )
    }

    pub fn edges_directed(
        &self,
        i: NodeIndex,
        dir: petgraph::Direction,
    ) -> Edges<f64, Directed, DefaultIx> {
        self.inner.edges_directed(i, dir)
    }
}

#[derive(Debug, Clone)]
pub struct InventoryItem {
    pub name: String,
    pub count: u32,
}

impl InventoryItem {
    pub fn new(name: &str, count: u32) -> InventoryItem {
        InventoryItem {
            name: name.into(),
            count,
        }
    }
}

#[derive(Debug, Clone)]
pub struct InventoryLocation {
    pub entity_name: String,
    pub position: Position,
    pub inventory_type: u32,
}

#[derive(Debug, Clone)]
pub struct EntityPlacement {
    pub item_name: String,
    pub position: Position,
    pub direction: Direction,
}

#[derive(Debug, Clone)]
pub struct PositionRadius {
    pub position: Position,
    pub radius: f64,
}
impl PositionRadius {
    pub fn new(x: f64, y: f64, radius: f64) -> PositionRadius {
        PositionRadius {
            position: Position::new(x, y),
            radius,
        }
    }
    pub fn from_position(pos: &Position, radius: f64) -> PositionRadius {
        PositionRadius {
            position: pos.clone(),
            radius,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MineTarget {
    pub position: Position,
    pub name: String,
    pub count: u32,
}
