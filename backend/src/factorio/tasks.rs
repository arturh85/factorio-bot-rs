use crate::types::{Direction, Position};
use actix::{Addr, SystemService};
use actix_taskqueue::queue::TaskQueue;
use actix_taskqueue::worker::*;
use petgraph::stable_graph::StableGraph;
use serde::export::Formatter;

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

pub type PlayerId = u32;

#[derive(Debug, Clone)]
pub enum TaskData {
    Mine(MineTarget),
    Walk(PositionRadius),
    Craft(InventoryItem),
    InsertToInventory(InventoryLocation, InventoryItem),
    RemoveFromInventory(InventoryLocation, InventoryItem),
    PlaceEntity(EntityPlacement),
}

#[derive(Default, Clone)]
pub struct Task {
    pub name: String,
    pub player_id: Option<u32>,
    pub data: Option<TaskData>,
}

impl Task {
    pub fn new(player_id: Option<u32>, name: &str, data: Option<TaskData>) -> Task {
        Task {
            name: name.into(),
            player_id,
            data,
        }
    }
    pub fn new_craft(player_id: u32, item: InventoryItem) -> Task {
        Task::new(
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
    pub fn new_walk(player_id: u32, target: PositionRadius) -> Task {
        Task::new(
            Some(player_id),
            &*format!("Walk to {}", target.position),
            Some(TaskData::Walk(target)),
        )
    }
    pub fn new_mine(player_id: u32, target: MineTarget) -> Task {
        Task::new(
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
    pub fn new_insert_to_inventory(
        player_id: u32,
        location: InventoryLocation,
        item: InventoryItem,
    ) -> Task {
        Task::new(
            Some(player_id),
            &*format!(
                "Insert {}x{} into {} at {}",
                &item.name, &item.count, location.entity_name, location.position
            ),
            Some(TaskData::InsertToInventory(location, item)),
        )
    }
}

impl std::fmt::Display for Task {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name)?;
        Ok(())
    }
}
impl std::fmt::Debug for Task {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name)?;
        Ok(())
    }
}

pub struct TaskResult(i32);

pub type TaskGraph = StableGraph<Task, f64>;

pub fn dotgraph(graph: &TaskGraph) -> String {
    use petgraph::dot::{Config, Dot};
    format!(
        "digraph {{\n{:?}}}\n",
        Dot::with_config(graph, &[Config::GraphContentOnly])
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_play() {
        let mut g = TaskGraph::new();
        let rocket_node = g.add_node(Task::new_craft(1, InventoryItem::new("rocket", 42)));
        let walk_node = g.add_node(Task::new_walk(1, PositionRadius::new(1., 5., 2.)));
        g.add_edge(rocket_node, walk_node, 4.);
        println!("{}", dotgraph(&g));
    }
}

#[async_trait]
impl QueueConsumer<Task, TaskResult> for TaskWorker<Task, TaskResult> {
    async fn execute(&self, _task: Task) -> Result<TaskResult, WorkerExecuteError> {
        // if let Some(data) = task.data {
        //     match data {
        //         TaskData::Craft((item_name, item_count)) => {}
        //         TaskData::Walk(position) => {}
        //     }
        // }

        // let Task(n) = task;
        // if n >= 5 {
        //     Ok(TaskResult(n + 5))
        // } else if n > 0 {
        //     Err(WorkerExecuteError::Retryable)
        // } else {
        //     Err(WorkerExecuteError::NonRetryable)
        // }
        Err(WorkerExecuteError::NonRetryable)
    }

    fn get_queue(&self) -> Addr<TaskQueue<Task>> {
        TaskQueue::<Task>::from_registry()
    }

    fn retry(&self, _task: Task) -> Task {
        // let Task(n) = task;
        // println!("RETRYING VALUE = {}", n);
        // Task(n + 1)

        Task::default()
    }

    fn drop(&self, _task: Task) {
        // let Task(n) = task;
        // println!("DROPPED TASK WITH VALUE = {}", n);
    }

    fn result(&self, _result: TaskResult) {
        // let TaskResult(n) = result;
        // println!("RESULT = {}", n);
    }
}
