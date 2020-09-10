use crate::factorio::instance_setup::setup_factorio_instance;
use crate::factorio::process_control::{start_factorio_server, FactorioStartCondition};
use crate::factorio::rcon::{FactorioRcon, RconSettings};
use crate::factorio::roll_best_seed::find_nearest_entities;
use crate::factorio::task_graph::{MineTarget, PositionRadius, TaskGraph, TaskNode, TaskResult};
use crate::factorio::util::calculate_distance;
use crate::factorio::world::{FactorioWorld, FactorioWorldWriter};
use crate::factorio::ws::FactorioWebSocketServer;
use crate::types::{
    Direction, EntityName, FactorioEntity, FactorioPlayer, PlayerChangedMainInventoryEvent,
    PlayerChangedPositionEvent, Position,
};
use actix::{Addr, SystemService};
use actix_taskqueue::messages::Push;
use actix_taskqueue::queue::TaskQueue;
use actix_taskqueue::worker::TaskWorker;
use async_std::sync::Arc;
use evmap::ReadGuard;
use noisy_float::types::R64;
use num_traits::ToPrimitive;
use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;
use std::collections::{BTreeMap, HashMap};
use std::time::Instant;

pub struct Planner {
    graph: TaskGraph,
    rcon: Arc<FactorioRcon>,
    real_world: Arc<FactorioWorld>,
    plan_world: FactorioWorldWriter,
}

impl Planner {
    pub async fn plan(
        &mut self,
        bot_count: u32,
    ) -> anyhow::Result<(TaskGraph, Arc<FactorioWorld>)> {
        let player_ids = self.initiate_missing_players_with_default_inventory(bot_count);
        let mut parent = self
            .graph
            .add_node(TaskNode::new(None, "Process Start", None));

        // mine 1x rock-huge
        parent = self
            .add_mine_entities_with_bots(
                parent,
                &player_ids,
                &Position::default(),
                Some("rock-huge".into()),
                None,
            )
            .await?;

        // mine 1x tree
        parent = self
            .add_mine_entities_with_bots(
                parent,
                &player_ids,
                &Position::default(),
                None,
                Some("tree".into()),
            )
            .await?;

        // build starter base
        parent = self.add_build_starter_base(parent, &player_ids).await?;
        self.plan_world.world.entity_graph.connect()?;

        let end = self
            .graph
            .add_node(TaskNode::new(None, "Process End", None));
        self.graph.add_edge(parent, end, 0.);
        Ok((self.graph.clone(), self.plan_world.world.clone()))
    }

    #[allow(clippy::ptr_arg)]
    pub async fn add_build_starter_base(
        &mut self,
        mut parent: NodeIndex,
        player_ids: &Vec<u32>,
    ) -> anyhow::Result<NodeIndex> {
        parent = self
            .graph
            .add_process_start_node(parent, "Build StarterBase");

        // build 2x iron minerFurnace
        parent = self
            .add_build_starter_miner_furnace(parent, &player_ids, "iron-ore", "iron-plate", 2)
            .await?;
        // // build 2x coal minerLoop
        // parent = self
        //     .add_build_starter_miner_loop(parent, &player_ids, "coal", 2)
        //     .await?;
        // build 2x iron minerFurnace
        parent = self
            .add_build_starter_miner_furnace(parent, &player_ids, "iron-ore", "iron-plate", 2)
            .await?;
        // // build 2x stone minerChest
        // parent = self
        //     .add_build_starter_miner_chest(parent, &player_ids, "stone", 2)
        //     .await?;

        Ok(parent)
    }

    #[allow(clippy::ptr_arg)]
    pub async fn add_build_starter_miner_furnace(
        &mut self,
        parent: NodeIndex,
        bots: &Vec<u32>,
        ore_name: &str,
        plate_name: &str,
        miner_furnace_count: u16,
    ) -> anyhow::Result<NodeIndex> {
        let start = self.graph.add_process_start_node(
            parent,
            &format!(
                "Build MinerFurnace for {} -> {} x {}",
                ore_name, plate_name, miner_furnace_count,
            ),
        );
        let mut weights: HashMap<NodeIndex, R64> = HashMap::new();
        let patches = self.plan_world.world.resource_patches(ore_name);
        if patches.is_empty() {
            return Err(anyhow!("no {} found", ore_name));
        }
        let patch = patches.first().unwrap();
        let rect = patch.find_free_rect(
            (miner_furnace_count * 2) as u32,
            (miner_furnace_count * 4) as u32,
            &Position::default(),
            &self.plan_world.world.blocked,
        );
        if rect.is_none() {
            return Err(anyhow!(
                "no free rect found for {} in patch of size {}",
                ore_name,
                patch.elements.len()
            ));
        }
        let rect = rect.unwrap();
        let mut bots = bots.clone();
        let entity_name = EntityName::BurnerMiningDrill.to_string();
        bots.sort_by(|a, b| {
            let da = self.inventory(*a, &entity_name);
            let db = self.inventory(*b, &entity_name);
            db.cmp(&da)
        });
        let mut to_place = miner_furnace_count;
        for player_id in &bots {
            let to_place_player =
                to_place.min((miner_furnace_count as f64 / bots.len() as f64).ceil() as u16);
            to_place -= to_place_player;
            if to_place_player == 0 {
                continue;
            }
            let player_parent = self.graph.add_node(TaskNode::new(
                Some(*player_id),
                &*format!(
                    "Bot #{} at {}",
                    player_id,
                    &self.player(*player_id).position
                ),
                None,
            ));
            let mut parent = player_parent;
            for i in 0..to_place_player {
                let miner_position = rect
                    .left_top
                    .add(&Position::new((to_place + i + 1) as f64 * 2., 1.));
                let furnace_position = miner_position.add(&Position::new(0., 2.));
                parent = self
                    .add_place(
                        parent,
                        *player_id,
                        FactorioEntity::new_stone_furnace(&furnace_position, Direction::North),
                    )
                    .await?;
                parent = self
                    .add_place(
                        parent,
                        *player_id,
                        FactorioEntity::new_burner_mining_drill(&miner_position, Direction::South),
                    )
                    .await?;
            }
            self.graph.add_edge(start, player_parent, 0.);
            weights.insert(parent, self.graph.weight(player_parent, parent));
        }
        // TODO: collect ingredients for (miner + furnace) * minerFurnaceCount
        // TODO: init process with coal?
        Ok(self.join_using_weights(start, weights))
    }

    #[allow(clippy::ptr_arg)]
    pub async fn add_build_starter_miner_chest(
        &mut self,
        parent: NodeIndex,
        _bots: &Vec<u32>,
        ore_name: &str,
        miner_chest_count: u16,
    ) -> anyhow::Result<NodeIndex> {
        let start = self.graph.add_process_start_node(
            parent,
            &format!("Build MinerChest for {} x {}", ore_name, miner_chest_count,),
        );
        let weights: HashMap<NodeIndex, R64> = HashMap::new();
        // TODO: find ore patch
        // TODO: collect ingredients for (miner + furnace) * minerFurnaceCount
        // TODO: place entities
        // TODO: init process with coal?
        Ok(self.join_using_weights(start, weights))
    }

    #[allow(clippy::ptr_arg)]
    pub async fn add_build_starter_miner_loop(
        &mut self,
        parent: NodeIndex,
        _bots: &Vec<u32>,
        ore_name: &str,
        miner_count: u16,
    ) -> anyhow::Result<NodeIndex> {
        let start = self.graph.add_process_start_node(
            parent,
            &format!("Build MinerLoop for {}  x {}", ore_name, miner_count,),
        );
        let weights: HashMap<NodeIndex, R64> = HashMap::new();
        // TODO: find ore patch
        // TODO: collect ingredients for (miner + furnace) * minerFurnaceCount
        // TODO: place entities
        // TODO: init process with coal?
        Ok(self.join_using_weights(start, weights))
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
        let start = self.graph.add_process_start_node(
            parent,
            &format!(
                "Mine {} with {} Bots",
                match name {
                    Some(ref name) => name.clone(),
                    None => entity_type
                        .as_ref()
                        .expect("must have name or entity_type")
                        .clone(),
                },
                bots.len()
            ),
        );
        let mut entities: Vec<FactorioEntity> =
            find_nearest_entities(self.rcon.clone(), search_center, 500., name, entity_type)
                .await?;

        let mut weights: HashMap<NodeIndex, R64> = HashMap::new();

        for player_id in bots {
            let player_parent = self.graph.add_node(TaskNode::new(
                Some(*player_id),
                &*format!(
                    "Bot #{} at {}",
                    player_id,
                    &self.player(*player_id).position
                ),
                None,
            ));
            let mut parent = player_parent;
            if !entities.is_empty() {
                let entity = entities.remove(0);
                parent = self
                    .add_mine(parent, *player_id, &entity.position, &entity.name, 1)
                    .await?
            }
            self.graph.add_edge(start, player_parent, 0.);
            weights.insert(parent, self.graph.weight(player_parent, parent));
        }
        Ok(self.join_using_weights(start, weights))
    }

    pub async fn add_walk(
        &mut self,
        parent_node: NodeIndex,
        player_id: u32,
        goal: &PositionRadius,
    ) -> anyhow::Result<NodeIndex> {
        let distance = self.distance(player_id, &goal.position);
        let task = TaskNode::new_walk(player_id, goal.clone());
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
        let task = TaskNode::new_mine(
            player_id,
            MineTarget {
                name: name.into(),
                count,
                position: position.clone(),
            },
        );
        let node = self.graph.add_node(task);

        let mut mining_time = 5.;
        let mut inventory = *self.player(player_id).main_inventory.clone();
        if let Some(prototype) = self.plan_world.world.entity_prototypes.get_one(name) {
            if let Some(result) = prototype.mine_result.as_ref() {
                for (mine_name, mine_count) in result {
                    if let Some(inventory_count) = inventory.get(mine_name) {
                        let cnt = *mine_count + *inventory_count;
                        inventory.insert(mine_name.clone(), cnt);
                    } else {
                        inventory.insert(mine_name.clone(), *mine_count);
                    }
                }
                if let Some(time) = prototype.mining_time.as_ref() {
                    mining_time = time.to_f64().unwrap().ceil()
                }
            }
        }
        self.plan_world
            .player_changed_main_inventory(PlayerChangedMainInventoryEvent {
                player_id,
                main_inventory: Box::new(inventory),
            })?;

        self.graph.add_edge(parent, node, mining_time);
        Ok(node)
    }
    pub async fn add_place(
        &mut self,
        parent: NodeIndex,
        player_id: u32,
        entity: FactorioEntity,
    ) -> anyhow::Result<NodeIndex> {
        let mut parent = parent;
        let player = self.player(player_id);
        let distance = calculate_distance(&player.position, &entity.position).ceil();
        let build_distance = player.build_distance as f64;
        drop(player);
        if distance > build_distance {
            parent = self
                .add_walk(
                    parent,
                    player_id,
                    &PositionRadius::from_position(&entity.position, build_distance),
                )
                .await?;
        }

        let mut inventory = *self.player(player_id).main_inventory.clone();
        let inventory_item_count = *inventory.get(&entity.name).unwrap_or(&0);
        if inventory_item_count < 1 {
            return Err(anyhow!(
                "player #{} does not have {} in inventory",
                player_id,
                &entity.name
            ));
        }
        let name = entity.name.clone();
        let task = TaskNode::new_place(player_id, entity.clone());
        let node = self.graph.add_node(task);
        inventory.insert(name, inventory_item_count - 1);

        self.plan_world
            .player_changed_main_inventory(PlayerChangedMainInventoryEvent {
                player_id,
                main_inventory: Box::new(inventory),
            })?;
        self.plan_world.on_some_entity_created(entity)?;
        self.graph.add_edge(parent, node, 1.);
        Ok(node)
    }

    pub fn new(world: Arc<FactorioWorld>, rcon: Arc<FactorioRcon>) -> Planner {
        let mut plan_world = FactorioWorldWriter::new();
        plan_world.import(world.clone()).expect("import failed");
        Planner {
            graph: TaskGraph::new(),
            rcon,
            real_world: world,
            plan_world,
        }
    }

    fn initiate_missing_players_with_default_inventory(&mut self, bot_count: u32) -> Vec<u32> {
        let mut player_ids: Vec<u32> = vec![];
        for player_id in 1u32..=bot_count {
            player_ids.push(player_id);
            // initialize missing players with default inventory
            if self.real_world.players.get_one(&player_id).is_none() {
                let mut main_inventory: BTreeMap<String, u32> = BTreeMap::new();
                main_inventory.insert(EntityName::Wood.to_string(), 1);
                main_inventory.insert(EntityName::StoneFurnace.to_string(), 1);
                main_inventory.insert(EntityName::BurnerMiningDrill.to_string(), 1);
                self.plan_world
                    .player_changed_main_inventory(PlayerChangedMainInventoryEvent {
                        player_id,
                        main_inventory: Box::new(main_inventory.clone()),
                    })
                    .expect("failed to set player inventory");
            }
        }
        player_ids
    }

    fn player(&self, player_id: u32) -> ReadGuard<FactorioPlayer> {
        self.plan_world
            .world
            .players
            .get_one(&player_id)
            .expect("failed to find player")
    }
    fn inventory(&self, player_id: u32, name: &str) -> u32 {
        *self
            .player(player_id)
            .main_inventory
            .get(name)
            .unwrap_or(&0)
    }
    fn distance(&self, player_id: u32, position: &Position) -> f64 {
        calculate_distance(&self.player(player_id).position, position).ceil()
    }

    fn join_using_weights(
        &mut self,
        start: NodeIndex,
        weights: HashMap<NodeIndex, R64>,
    ) -> NodeIndex {
        let end = self.graph.add_process_end_node();
        let max_weight = weights.values().max();
        if let Some(max_weight) = max_weight {
            for (node, weight) in weights.iter() {
                self.graph
                    .add_edge(*node, end, (*max_weight - *weight).to_f64().unwrap());
            }
        } else {
            self.graph.add_edge(start, end, 0.);
        }
        end
    }
}

pub async fn start_factorio_and_plan_graph(
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
        true,
        map_exchange_string,
        seed,
        true,
    )
    .await
    .expect("failed to initially setup instance");

    let (world, rcon, mut child) = start_factorio_server(
        &workspace_path,
        &rcon_settings,
        None,
        instance_name,
        None,
        false,
        true,
        FactorioStartCondition::DiscoveryComplete,
    )
    .await
    .expect("failed to start");
    let mut planner = Planner::new(world, rcon);
    let (graph, world) = planner.plan(bot_count).await?;
    if let Some(players) = &world.players.read() {
        for (player_id, player) in players {
            if let Some(player) = player.get_one() {
                info!(
                    "bot #{} endet up at {} with inventory: {:?}",
                    player_id, player.position, player.main_inventory
                );
            }
        }
    }
    // if let Some(resources) = &world.resources.read() {
    //     for (name, _) in resources {
    //         let patches = world.resource_patches(&name);
    //         for patch in patches {
    //             info!(
    //                 "{} patch at {} size {}",
    //                 patch.name,
    //                 patch.rect.center(),
    //                 patch.elements.len()
    //             );
    //         }
    //     }
    // }

    let process_start = graph.node_indices().next().unwrap();
    let process_end = graph.node_indices().last().unwrap();
    let (weight, _) = graph
        .astar(process_start, process_end)
        .expect("no path found");
    info!("shortest path: {}", weight);

    // println!("{}", dotgraph_task(&graph));
    // println!("{}", dotgraph_entity(&entity_graph));

    child.kill().expect("failed to kill child");
    info!("took <yellow>{:?}</>", started.elapsed());
    Ok(graph)
}

// pub async fn execute_node(node: NodeIndex<u32>) -> JoinHandle<NodeIndex<u32>> {}

pub fn execute_plan(
    _world: Arc<FactorioWorld>,
    _rcon: Arc<FactorioRcon>,
    _websocket_server: Option<Addr<FactorioWebSocketServer>>,
    plan: TaskGraph,
) {
    let queue = TaskQueue::<NodeIndex>::from_registry();
    let _worker = TaskWorker::<NodeIndex, TaskResult>::new();

    let root = plan.node_indices().next().unwrap();

    let pointer = root;
    let _tick = 0;
    loop {
        // if let Some(websocket_server) = websocket_server.as_ref() {
        //     websocket_server
        //         .send(TaskStarted {
        //             node_id: pointer.index(),
        //             tick,
        //         })
        //         .await?;
        // }

        // let incoming = plan.edges_directed(pointer, petgraph::Direction::Incoming);
        // for edge in incoming {
        //     let target = edge.target();
        // }
        let outgoing = plan.edges_directed(pointer, petgraph::Direction::Outgoing);
        for edge in outgoing {
            queue.do_send(Push::new(edge.target()));
        }

        // let foo = worker.next().await;

        // let task = plan.node_weight_mut(pointer).unwrap();
        // if task.data.is_some() {
        //     queue.do_send(Push::new(pointer))
        // }
    }
}
