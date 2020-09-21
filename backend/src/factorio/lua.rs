use std::env;
use std::fs::read_to_string;
use std::path::Path;
use std::sync::Arc;

use dashmap::lock::RwLock;
use num_traits::ToPrimitive;
use rlua::{Context, Lua, Table};

use crate::factorio::task_graph::{MineTarget, PositionRadius, TaskGraph};
use crate::factorio::util::calculate_distance;
use crate::factorio::world::FactorioWorld;
use crate::types::{
    FactorioEntity, FactorioPlayer, PlayerChangedMainInventoryEvent, PlayerChangedPositionEvent,
    Position,
};

pub struct LuaFactorioWorld {}

impl LuaFactorioWorld {
    pub fn create(ctx: Context, _world: Arc<FactorioWorld>) -> Result<Table, rlua::Error> {
        let map_table = ctx.create_table()?;

        let world = _world.clone();
        map_table.set(
            "recipe",
            ctx.create_function(move |ctx, name: String| match world.recipes.get(&name) {
                Some(recipe) => Ok(rlua_serde::to_value(ctx, recipe.clone())),
                None => Err(rlua::Error::RuntimeError("recipe not found".into())),
            })?,
        )?;

        let world = _world.clone();
        map_table.set(
            "player",
            ctx.create_function(
                move |ctx, player_id: u32| match world.players.get(&player_id) {
                    Some(player) => Ok(rlua_serde::to_value(ctx, player.clone())),
                    None => Err(rlua::Error::RuntimeError("player not found".into())),
                },
            )?,
        )?;

        let world = _world;
        map_table.set(
            "inventory",
            ctx.create_function(move |_ctx, (player_id, item_name): (u32, String)| {
                match world.players.get(&player_id) {
                    Some(player) => match player.main_inventory.get(&item_name) {
                        Some(cnt) => Ok(*cnt),
                        None => Ok(0),
                    },
                    None => Err(rlua::Error::RuntimeError("player not found".into())),
                }
            })?,
        )?;

        Ok(map_table)
    }
}

// pub struct LuaFactorioRcon {
//     inner: Arc<FactorioRcon>,
//     world: Arc<FactorioWorld>,
// }
//
// impl LuaFactorioRcon {
//     pub fn new(world: Arc<FactorioWorld>, rcon: Arc<FactorioRcon>) -> LuaFactorioRcon {
//         LuaFactorioRcon { inner: rcon, world }
//     }
// }

pub struct PlanBuilder {
    graph: Arc<RwLock<TaskGraph>>,
    world: Arc<FactorioWorld>,
}
impl PlanBuilder {
    pub fn new(graph: Arc<RwLock<TaskGraph>>, world: Arc<FactorioWorld>) -> PlanBuilder {
        PlanBuilder { graph, world }
    }

    pub fn mine(
        &self,
        player_id: u32,
        position: Position,
        name: &str,
        count: u32,
    ) -> anyhow::Result<()> {
        let mut graph = self.graph.write();
        let player = self.world.players.get(&player_id).unwrap();
        let distance = calculate_distance(&player.position, &position).ceil();
        let reach_distance = player.resource_reach_distance as f64;
        if distance > reach_distance {
            graph.add_walk_node(
                player_id,
                distance,
                PositionRadius::from_position(&position, reach_distance),
            );
        }
        let mut mining_time = 5.;
        let mut inventory = (*player.main_inventory).clone();
        if let Some(prototype) = self.world.entity_prototypes.get(name) {
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
        graph.add_mine_node(
            player_id,
            mining_time,
            MineTarget {
                name: name.into(),
                count,
                position,
            },
        );
        drop(player);
        self.world
            .player_changed_main_inventory(PlayerChangedMainInventoryEvent {
                player_id,
                main_inventory: Box::new(inventory),
            })?;
        Ok(())
    }

    fn distance(&self, player_id: u32, position: &Position) -> f64 {
        calculate_distance(
            &self.world.players.get(&player_id).unwrap().position,
            position,
        )
        .ceil()
    }

    fn player(&self, player_id: u32) -> FactorioPlayer {
        self.world
            .players
            .get(&player_id)
            .expect("failed to find player")
            .clone()
    }
    // fn inventory(&self, player_id: u32, name: &str) -> u32 {
    //     *self
    //         .player(player_id)
    //         .main_inventory
    //         .get(name)
    //         .unwrap_or(&0)
    // }

    pub fn add_walk(&self, player_id: u32, goal: PositionRadius) -> anyhow::Result<()> {
        let distance = self.distance(player_id, &goal.position);
        let mut graph = self.graph.write();
        self.world
            .player_changed_position(PlayerChangedPositionEvent {
                player_id,
                position: goal.position.clone(),
            })?;
        graph.add_walk_node(player_id, distance, goal);
        Ok(())
    }

    pub fn add_place(&mut self, player_id: u32, entity: FactorioEntity) -> anyhow::Result<()> {
        let player = self.player(player_id);
        let distance = calculate_distance(&player.position, &entity.position);
        let build_distance = player.build_distance as f64;
        if distance > build_distance {
            self.add_walk(
                player_id,
                PositionRadius::from_position(&entity.position, build_distance),
            )?;
        }
        let mut inventory = *self.player(player_id).main_inventory;
        let inventory_item_count = *inventory.get(&entity.name).unwrap_or(&0);
        if inventory_item_count < 1 {
            return Err(anyhow!(
                "player #{} does not have {} in inventory",
                player_id,
                &entity.name
            ));
        }
        let mut graph = self.graph.write();
        graph.add_place_node(player_id, 1., entity.clone());
        inventory.insert(entity.name.clone(), inventory_item_count - 1);
        self.world
            .player_changed_main_inventory(PlayerChangedMainInventoryEvent {
                player_id,
                main_inventory: Box::new(inventory),
            })?;
        self.world.on_some_entity_created(entity)?;
        Ok(())
    }

    pub fn group_start(&self, label: &str) {
        let mut graph = self.graph.write();
        graph.group_start(label);
    }

    pub fn group_end(&self) {
        let mut graph = self.graph.write();
        graph.group_end();
    }
}

pub struct LuaPlanBuilder {}
impl LuaPlanBuilder {
    pub fn create(ctx: Context, plan_builder: PlanBuilder) -> Result<Table, rlua::Error> {
        let map_table = ctx.create_table()?;
        let _plan_builder = Arc::new(plan_builder);

        let plan_builder = _plan_builder.clone();
        map_table.set(
            "mine",
            ctx.create_function(
                move |_ctx, (player_id, position, name, count): (u32, String, String, u32)| {
                    plan_builder
                        .mine(player_id, position.parse().unwrap(), name.as_str(), count)
                        .unwrap();
                    Ok(())
                },
            )?,
        )?;
        let plan_builder = _plan_builder.clone();
        map_table.set(
            "groupStart",
            ctx.create_function(move |_ctx, label: String| {
                plan_builder.group_start(label.as_str());
                Ok(())
            })?,
        )?;
        let plan_builder = _plan_builder;
        map_table.set(
            "groupEnd",
            ctx.create_function(move |_ctx, ()| {
                plan_builder.group_end();
                Ok(())
            })?,
        )?;
        Ok(map_table)
    }
}

pub fn execute_lua_plan(plan_name: &str) -> anyhow::Result<()> {
    let lua_path_str = format!("../plans/{}.lua", plan_name);
    let lua_path = Path::new(&lua_path_str);

    if !lua_path.exists() {
        let path = env::current_dir()?;
        println!("The current directory is {}", path.display());
        anyhow::bail!("plan {} not found at {}", plan_name, lua_path_str);
    }
    let lua_code = read_to_string(lua_path)?;
    let lua = Lua::new();

    let players: Vec<FactorioPlayer> = vec![
        FactorioPlayer {
            player_id: 1,
            ..Default::default()
        },
        FactorioPlayer {
            player_id: 2,
            ..Default::default()
        },
        FactorioPlayer {
            player_id: 3,
            ..Default::default()
        },
        FactorioPlayer {
            player_id: 4,
            ..Default::default()
        },
    ];
    // let players: Vec<String> = vec!["a".into()];

    lua.context(|lua_ctx| {
        let globals = lua_ctx.globals();
        globals
            .set("all_bots", rlua_serde::to_value(lua_ctx, &players).unwrap())
            .unwrap();
    });
    lua.context(|lua_context| lua_context.load(&lua_code).exec())?;

    // lua.context(|lua_ctx: Context| {
    //     // You can get and set global variables.  Notice that the globals table here is a permanent
    //     // reference to _G, and it is mutated behind the scenes as Lua code is loaded.  This API is
    //     // based heavily around sharing and internal mutation (just like Lua itself).
    //
    //     let globals = lua_ctx.globals();
    //
    //     globals.set("string_var", "hello")?;
    //     globals.set("int_var", 42)?;
    //
    //     let check_equal =
    //         lua_ctx.create_function(|_, (list1, list2): (Vec<String>, Vec<String>)| {
    //             // This function just checks whether two string lists are equal, and in an inefficient way.
    //             // Lua callbacks return `rlua::Result`, an Ok value is a normal return, and an Err return
    //             // turns into a Lua 'error'.  Again, any type that is convertible to Lua may be returned.
    //             Ok(list1 == list2)
    //         })?;
    //     globals.set("check_equal", check_equal)?;
    //
    //     lua_ctx
    //         .load(
    //             r#"
    //             global = 'foo'..'bar'
    //         "#,
    //         )
    //         .set_name("example code")?
    //         .exec()?;
    //
    //     Ok(())
    // })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn test_lua_plan() {
        // execute_lua_plan("arturh").unwrap();
    }
}
