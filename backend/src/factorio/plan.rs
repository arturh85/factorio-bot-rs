use crate::factorio::instance_setup::setup_factorio_instance;
use crate::factorio::process_control::start_factorio_server;
use crate::factorio::rcon::{FactorioRcon, RconSettings};
use crate::factorio::roll_best_seed::find_nearest_entities;
use crate::factorio::tasks::{dotgraph, MineTarget, PositionRadius, Task, TaskGraph};
use crate::factorio::util::calculate_distance;
use crate::factorio::world::{FactorioWorld, FactorioWorldWriter};
use crate::types::{
    FactorioEntity, FactorioPlayer, PlayerChangedMainInventoryEvent, PlayerChangedPositionEvent,
    Position,
};
use async_std::sync::Arc;
use evmap::ReadGuard;
use petgraph::graph::NodeIndex;
use std::collections::BTreeMap;
use std::time::Instant;

pub struct Planner {
    graph: TaskGraph,
    rcon: Arc<FactorioRcon>,
    world: Arc<FactorioWorld>,
    plan_world: FactorioWorldWriter,
}

impl Planner {
    pub fn new(world: Arc<FactorioWorld>, rcon: Arc<FactorioRcon>) -> Planner {
        let mut plan_world = FactorioWorldWriter::new();
        plan_world.import(world.clone()).expect("import failed");
        Planner {
            graph: TaskGraph::new(),
            rcon,
            world,
            plan_world,
        }
    }

    fn player(&self, player_id: u32) -> ReadGuard<FactorioPlayer> {
        self.plan_world
            .world
            .players
            .get_one(&player_id)
            .expect("failed to find player")
    }

    fn distance(&self, player_id: u32, position: &Position) -> f64 {
        calculate_distance(&self.player(player_id).position, position).ceil()
    }

    pub async fn add_walk(
        &mut self,
        parent_node: NodeIndex,
        player_id: u32,
        goal: &PositionRadius,
    ) -> anyhow::Result<NodeIndex> {
        let distance = self.distance(player_id, &goal.position);
        let task = Task::new_walk(player_id, goal.clone());
        let node = self.graph.add_node(task);
        self.graph.add_edge(parent_node, node, distance);
        self.plan_world
            .player_changed_position(PlayerChangedPositionEvent {
                player_id,
                position: goal.position.clone(),
            })?;
        Ok(node)
    }

    pub async fn add_mine(
        &mut self,
        parent: NodeIndex,
        player_id: u32,
        position: &Position,
        name: &str,
        count: u32,
    ) -> anyhow::Result<NodeIndex> {
        let mut parent = parent;
        let player = self.player(player_id);
        let distance = calculate_distance(&player.position, position).ceil();
        let reach_distance = player.resource_reach_distance as f64;
        drop(player);
        if distance > reach_distance {
            parent = self
                .add_walk(
                    parent,
                    player_id,
                    &PositionRadius::from_position(&position, reach_distance),
                )
                .await?;
        }
        let task = Task::new_mine(
            player_id,
            MineTarget {
                name: name.into(),
                count,
                position: position.clone(),
            },
        );
        let node = self.graph.add_node(task);
        self.graph.add_edge(parent, node, 5.);
        Ok(node)
    }

    #[allow(clippy::ptr_arg)]
    pub async fn add_mine_entities_with_bots(
        &mut self,
        parent: NodeIndex,
        bots: &Vec<u32>,
        search_center: &Position,
        name: Option<String>,
        entity_type: Option<String>,
    ) -> anyhow::Result<NodeIndex> {
        let start = self.graph.add_node(Task::new(
            None,
            &format!(
                "Start: Mine {} with {} Bots",
                match name {
                    Some(ref name) => name.clone(),
                    None => entity_type
                        .as_ref()
                        .expect("must have name or entity_type")
                        .clone(),
                },
                bots.len()
            ),
            None,
        ));
        self.graph.add_edge(parent, start, 0.);
        let end = self.graph.add_node(Task::new(None, "End", None));

        let mut entities: Vec<FactorioEntity> =
            find_nearest_entities(self.rcon.clone(), search_center, name, entity_type).await?;

        for player_id in bots {
            let player_parent = self.graph.add_node(Task::new(
                Some(*player_id),
                &*format!(
                    "Bot #{} at {}",
                    player_id,
                    &self.player(*player_id).position
                ),
                None,
            ));
            self.graph.add_edge(start, player_parent, 0.);
            let mut parent = player_parent;
            if !entities.is_empty() {
                let entity = entities.remove(0);
                parent = self
                    .add_mine(parent, *player_id, &entity.position, &entity.name, 1)
                    .await?
            }
            self.graph.add_edge(parent, end, 0.);
        }
        Ok(end)
    }

    #[allow(unused_assignments)]
    pub async fn plan(&mut self, bot_count: u32) -> anyhow::Result<TaskGraph> {
        let mut player_ids: Vec<u32> = vec![];

        for player_id in 1u32..=bot_count {
            player_ids.push(player_id);
            if self.world.players.get_one(&player_id).is_none() {
                let mut main_inventory: BTreeMap<String, u32> = BTreeMap::new();
                main_inventory.insert("wood".into(), 1);
                main_inventory.insert("stone-furnace".into(), 1);
                main_inventory.insert("burner-mining-drill".into(), 1);
                self.plan_world
                    .player_changed_main_inventory(PlayerChangedMainInventoryEvent {
                        player_id,
                        main_inventory: Box::new(main_inventory.clone()),
                    })?;
            }
        }

        let mut parent = self.graph.add_node(Task::new(None, "Process Start", None));
        parent = self
            .add_mine_entities_with_bots(
                parent,
                &player_ids,
                &Position::default(),
                Some("rock-huge".into()),
                None,
            )
            .await?;
        parent = self
            .add_mine_entities_with_bots(
                parent,
                &player_ids,
                &Position::default(),
                None,
                Some("tree".into()),
            )
            .await?;

        let end = self.graph.add_node(Task::new(None, "Process End", None));
        self.graph.add_edge(parent, end, 0.);

        Ok(self.graph.clone())
    }
}

pub async fn plan_graph(
    settings: config::Config,
    map_exchange_string: Option<&str>,
    seed: Option<&str>,
    bot_count: u32,
) -> anyhow::Result<TaskGraph> {
    let started = Instant::now();
    let instance_name = "plan";
    let workspace_path: String = settings.get("workspace_path")?;
    let rcon_settings = RconSettings::new(&settings, None);
    setup_factorio_instance(
        &workspace_path,
        &rcon_settings,
        None,
        instance_name,
        true,
        false,
        false,
        map_exchange_string,
        seed,
        true,
    )
    .await
    .expect("failed to initially setup instance");

    let (world, mut child) = start_factorio_server(
        &workspace_path,
        &rcon_settings,
        None,
        instance_name,
        None,
        false,
        true,
    )
    .await
    .expect("failed to start");
    let rcon = FactorioRcon::new(&rcon_settings, true)
        .await
        .expect("failed to rcon");
    rcon.silent_print("").await.expect("failed to silent print");
    let mut planner = Planner::new(world, Arc::new(rcon));
    let graph = planner.plan(bot_count).await?;
    println!("{}", dotgraph(&graph));
    child.kill().expect("failed to kill child");
    info!("took <yellow>{:?}</>", started.elapsed());
    Ok(graph)
}
