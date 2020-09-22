use crate::factorio::instance_setup::setup_factorio_instance;
use crate::factorio::plan_builder::create_lua_plan_builder;
use crate::factorio::process_control::{start_factorio_server, FactorioStartCondition};
use crate::factorio::rcon::{create_lua_rcon, FactorioRcon, RconSettings};
use crate::factorio::task_graph::TaskGraph;
use crate::factorio::world::{create_lua_world, FactorioWorld};
use crate::factorio::ws::FactorioWebSocketServer;
use crate::types::{EntityName, PlayerChangedMainInventoryEvent};
use actix::Addr;
use async_std::sync::Arc;
use dashmap::lock::RwLock;
use rlua::Lua;
use std::collections::BTreeMap;
use std::fs::read_to_string;
use std::path::Path;
use std::time::Instant;

pub struct Planner {
    rcon: Option<Arc<FactorioRcon>>,
    real_world: Arc<FactorioWorld>,
    plan_world: Arc<FactorioWorld>,
    graph: Arc<RwLock<TaskGraph>>,
}

impl Planner {
    pub fn new(world: Arc<FactorioWorld>, rcon: Option<Arc<FactorioRcon>>) -> Planner {
        let plan_world = FactorioWorld::new();
        plan_world.import(world.clone()).unwrap();
        Planner {
            graph: Arc::new(RwLock::new(TaskGraph::new())),
            rcon,
            real_world: world,
            plan_world: Arc::new(plan_world),
        }
    }

    pub async fn plan(&mut self, lua_code: &str, bot_count: u32) -> anyhow::Result<()> {
        let all_bots = self.initiate_missing_players_with_default_inventory(bot_count);
        self.plan_world.import(self.real_world.clone())?;
        let lua = Lua::new();
        // use rlua_async::ChunkExt;
        // use std::future::Future;
        // use std::pin::Pin;
        // lua.context::<_, Pin<Box<dyn Future<Output = rlua::Result<()>>>>>(|ctx| {
        lua.context::<_, rlua::Result<()>>(|ctx| {
            let world = create_lua_world(ctx, self.plan_world.clone()).unwrap();
            let plan =
                create_lua_plan_builder(ctx, self.graph.clone(), self.plan_world.clone()).unwrap();
            let globals = ctx.globals();
            globals.set("all_bots", all_bots).unwrap();
            globals.set("world", world).unwrap();
            globals.set("plan", plan).unwrap();
            if let Some(rcon) = self.rcon.as_ref() {
                let rcon = create_lua_rcon(ctx, rcon.clone()).unwrap();
                globals.set("rcon", rcon).unwrap();
            }
            let chunk = ctx.load(&lua_code);
            // chunk.exec_async(ctx)
            chunk.exec()
        })?;
        Ok(())
    }

    pub fn world(&self) -> Arc<FactorioWorld> {
        self.plan_world.clone()
    }
    pub fn graph(&self) -> TaskGraph {
        self.graph.read().clone()
    }

    fn initiate_missing_players_with_default_inventory(&mut self, bot_count: u32) -> Vec<u32> {
        let mut player_ids: Vec<u32> = vec![];
        for player_id in 1u32..=bot_count {
            player_ids.push(player_id);
            // initialize missing players with default inventory
            if self.real_world.players.get(&player_id).is_none() {
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
}

pub async fn start_factorio_and_plan_graph(
    settings: config::Config,
    map_exchange_string: Option<&str>,
    seed: Option<&str>,
    plan_name: &str,
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

    let mut planner = Planner::new(world, Some(rcon));
    let lua_path_str = format!("plans/{}.lua", plan_name);
    let lua_path = Path::new(&lua_path_str);
    let lua_path = std::fs::canonicalize(lua_path)?;
    if !lua_path.exists() {
        anyhow::bail!("plan {} not found at {}", plan_name, lua_path_str);
    }
    let lua_code = read_to_string(lua_path)?;
    planner.plan(lua_code.as_str(), bot_count).await?;
    let world = planner.world();
    let graph = planner.graph();
    for player in world.players.iter() {
        info!(
            "bot #{} endet up at {} with inventory: {:?}",
            player.player_id, player.position, player.main_inventory
        );
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

    world.entity_graph.connect().unwrap();
    world.flow_graph.update().unwrap();
    graph.print();
    println!("{}", graph.graphviz_dot());
    println!("{}", world.entity_graph.graphviz_dot());
    println!("{}", world.flow_graph.graphviz_dot());

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
    // let queue = TaskQueue::<NodeIndex>::from_registry();
    // let _worker = TaskWorker::<NodeIndex, TaskResult>::new();

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
        for _edge in outgoing {
            // queue.do_send(Push::new(edge.target()));
        }

        // let foo = worker.next().await;

        // let task = plan.node_weight_mut(pointer).unwrap();
        // if task.data.is_some() {
        //     queue.do_send(Push::new(pointer))
        // }
    }
}

#[cfg(test)]
mod tests {
    use crate::factorio::tests::{draw_world, fixture_world};

    use super::*;

    #[actix_rt::test]
    async fn test_planner() {
        let world = Arc::new(fixture_world());
        draw_world(world.clone());
        let mut planner = Planner::new(world, None);
        planner
            .plan(
                r##"
    plan.groupStart("Mine Stuff")
    for idx,playerId in pairs(all_bots) do
        plan.mine(playerId, "42,43", "rock-huge", 1)
    end
    plan.groupEnd("foo")
        "##,
                4,
            )
            .await
            .unwrap();
        let graph = planner.graph();
        assert_eq!(
            graph.graphviz_dot(),
            r#"digraph {
    0 [ label = "Process Start" ]
    1 [ label = "Process End" ]
    10 [ label = "Mining rock-huge" ]
    11 [ label = "End" ]
    2 [ label = "Start: Mine Stuff" ]
    3 [ label = "Walk to [42, 43]" ]
    4 [ label = "Mining rock-huge" ]
    5 [ label = "Walk to [42, 43]" ]
    6 [ label = "Mining rock-huge" ]
    7 [ label = "Walk to [42, 43]" ]
    8 [ label = "Mining rock-huge" ]
    9 [ label = "Walk to [42, 43]" ]
    0 -> 2 [ label = "0" ]
    10 -> 11 [ label = "0" ]
    11 -> 1 [ label = "0" ]
    2 -> 3 [ label = "61" ]
    2 -> 5 [ label = "61" ]
    2 -> 7 [ label = "61" ]
    2 -> 9 [ label = "61" ]
    3 -> 4 [ label = "3" ]
    4 -> 11 [ label = "0" ]
    5 -> 6 [ label = "3" ]
    6 -> 11 [ label = "0" ]
    7 -> 8 [ label = "3" ]
    8 -> 11 [ label = "0" ]
    9 -> 10 [ label = "3" ]
}
"#,
        );
    }
}
