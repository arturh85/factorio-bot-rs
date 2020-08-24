use crate::factorio::output_parser::FactorioWorld;
use crate::factorio::util::{
    blueprint_build_area, calculate_distance, hashmap_to_lua, move_position, position_in_rect,
    position_to_lua, rect_to_lua, str_to_lua, value_to_lua, vec_to_lua,
};
use crate::num_traits::FromPrimitive;
use crate::types::{
    Direction, FactorioEntity, FactorioForce, FactorioTile, InventoryResponse, Position, Rect,
    RequestEntity,
};
use config::Config;
use rcon::Connection;
use rocket::State;
use serde_json::Value;
use std::collections::HashMap;
use std::ops::Add;
use std::sync::Arc;
use unicode_segmentation::UnicodeSegmentation;

const RCON_INTERFACE: &str = "botbridge";

pub struct FactorioRcon {
    // conn: Mutex<Connection>,
    pool: bb8::Pool<ConnectionManager>,
    silent: bool,
}

pub struct ConnectionManager {
    address: String,
    pass: String,
}

unsafe impl Sync for ConnectionManager {}

impl ConnectionManager {
    pub fn new<S: Into<String>>(address: S, pass: S) -> Self {
        ConnectionManager {
            address: address.into(),
            pass: pass.into(),
        }
    }
}

#[async_trait]
impl bb8::ManageConnection for ConnectionManager {
    type Connection = rcon::Connection;
    type Error = rcon::Error;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        Connection::builder()
            .enable_factorio_quirks(true)
            .connect(&self.address, &self.pass)
            .await
    }

    async fn is_valid(&self, conn: Self::Connection) -> Result<Self::Connection, Self::Error> {
        Ok(conn)
    }

    fn has_broken(&self, _conn: &mut Self::Connection) -> bool {
        false
    }
}

impl FactorioRcon {
    pub async fn new(
        settings: &Config,
        server_host: Option<&str>,
        silent: bool,
    ) -> anyhow::Result<Self> {
        let rcon_port: String = settings.get("rcon_port").unwrap();
        let rcon_pass: String = settings.get("rcon_pass").unwrap();
        let address = format!("{}:{}", server_host.unwrap_or("127.0.0.1"), rcon_port);
        let manager = ConnectionManager::new(&address, &rcon_pass);
        Ok(FactorioRcon {
            pool: bb8::Pool::builder().max_size(15).build(manager).await?,
            silent,
        })
    }

    pub async fn send(&self, command: &str) -> anyhow::Result<Option<Vec<String>>> {
        if !self.silent {
            info!("<cyan>rcon</>  ⮜ <green>{}</>", command);
        }
        // let started = Instant::now();
        let mut conn = self.pool.get().await?;
        let result = conn.cmd(&String::from(command).add("\n")).await?;
        drop(conn);
        // info!("send took {} ms", started.elapsed().as_millis());
        if !result.is_empty() {
            if !self.silent {
                info!(
                    "<cyan>rcon</>  ⮞ <green>{}</>",
                    &result[0..result.len() - 1]
                );
            }
            Ok(Some(
                result[0..result.len() - 1]
                    .split('\n')
                    .map(|str| str.to_string())
                    .collect(),
            ))
        } else {
            Ok(None)
        }
    }

    async fn remote_call(
        &self,
        function_name: &str,
        args: Vec<&str>,
    ) -> anyhow::Result<Option<Vec<String>>> {
        let mut arg_string: String = args.join(", ");
        if !arg_string.is_empty() {
            arg_string = String::from(", ") + &arg_string;
        }
        self.send(&format!(
            "/silent-command remote.call('{}', '{}'{})",
            RCON_INTERFACE, function_name, arg_string
        ))
        .await
    }

    pub async fn print(&self, str: &str) -> anyhow::Result<()> {
        self.send(&format!("/c print({})", str_to_lua(str))).await?;
        Ok(())
    }

    pub async fn screenshot(&self, width: i16, height: i16, depth: i8) -> anyhow::Result<()> {
        self.send(&format!(
            "/screenshot {} {} {}",
            width.to_string(),
            height.to_string(),
            depth.to_string()
        ))
        .await?;
        Ok(())
    }

    pub async fn silent_print(&self, str: &str) -> anyhow::Result<()> {
        self.send(&format!("/silent-command print({})", str_to_lua(str)))
            .await?;
        Ok(())
    }

    pub async fn server_save(&self) -> anyhow::Result<()> {
        self.send("/server-save").await?;
        Ok(())
    }

    pub async fn whoami(&self, name: &str) -> anyhow::Result<()> {
        self.remote_call("whoami", vec![&str_to_lua(name)]).await?;
        Ok(())
    }

    pub async fn add_research(&self, technology_name: &str) -> anyhow::Result<()> {
        self.remote_call("add_research", vec![&str_to_lua(technology_name)])
            .await?;
        Ok(())
    }

    pub async fn cheat_item(
        &self,
        player_id: u32,
        item_name: &str,
        item_count: u32,
    ) -> anyhow::Result<()> {
        self.remote_call(
            "cheat_item",
            vec![
                &player_id.to_string(),
                &str_to_lua(item_name),
                &item_count.to_string(),
            ],
        )
        .await?;
        Ok(())
    }

    pub async fn cheat_technology(&self, technology_name: &str) -> anyhow::Result<()> {
        self.remote_call("cheat_technology", vec![&str_to_lua(technology_name)])
            .await?;
        Ok(())
    }

    pub async fn cheat_all_technologies(&self) -> anyhow::Result<()> {
        self.remote_call("cheat_all_technologies", vec![]).await?;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn place_blueprint(
        &self,
        player_id: u32,
        blueprint: String,
        position: &Position,
        direction: u8,
        force_build: bool,
        only_ghosts: bool,
        world: &State<'_, Arc<FactorioWorld>>,
    ) -> anyhow::Result<Vec<FactorioEntity>> {
        let player = world.players.get_one(&player_id).unwrap();
        let distance = calculate_distance(&player.position, &position);
        let build_distance = player.build_distance.unwrap_or(10) as f64;
        drop(player); // wow, without this factorio (?) freezes (!)
        if distance > build_distance {
            warn!("too far away, moving first!");
            self.move_player(world, player_id, position, Some(build_distance))
                .await?;
        }

        let build_area = blueprint_build_area(&world.entity_prototypes, &blueprint);
        let width_2 = build_area.width() / 2.0;
        let height_2 = build_area.height() / 2.0;
        let build_area = Rect {
            left_top: Position::new(position.x() - width_2, position.y() - height_2),
            right_bottom: Position::new(position.x() + width_2, position.y() + height_2),
        };
        let build_area_entities = self
            .find_entities_filtered(Some(build_area.clone()), None, None, None, None)
            .await?;

        for entity in build_area_entities {
            if entity.name != "character" && position_in_rect(&build_area, &entity.position) {
                warn!(
                    "mining entity in build area: {} @ {}/{}",
                    entity.name,
                    entity.position.x(),
                    entity.position.y()
                );
                self.player_mine(world, player_id, &entity.name, &entity.position, 1)
                    .await?;
            }
        }

        let lines = self
            .remote_call(
                "place_blueprint",
                vec![
                    &player_id.to_string(),
                    &str_to_lua(&blueprint),
                    &position.x().to_string(),
                    &position.y().to_string(),
                    &direction.to_string(),
                    &force_build.to_string(),
                    &only_ghosts.to_string(),
                ],
            )
            .await?;
        if lines.is_none() {
            return Err(anyhow!("Unexpected Empty Response"));
        }
        let mut json = lines.unwrap().pop().unwrap();
        // empty objects/arrays are the same in lua
        if json == "{}" {
            json = String::from("[]");
        }
        if &json[0..1] == "[" {
            Ok(serde_json::from_str(json.as_str())?)
        } else {
            Err(anyhow!("{}", json))
        }
    }

    pub async fn revive_ghost(
        &self,
        player_id: u32,
        name: &str,
        position: &Position,
        world: &State<'_, Arc<FactorioWorld>>,
    ) -> anyhow::Result<FactorioEntity> {
        let player = world.players.get_one(&player_id).unwrap();
        let build_distance = player.build_distance.unwrap_or(10) as f64;
        let distance = calculate_distance(&player.position, &position);
        drop(player); // wow, without this factorio (?) freezes (!)
        if distance > build_distance {
            warn!("too far away, moving first!");
            self.move_player(world, player_id, position, Some(build_distance))
                .await?;
        }
        let lines = self
            .remote_call(
                "revive_ghost",
                vec![
                    &player_id.to_string(),
                    &str_to_lua(&name),
                    &position.x().to_string(),
                    &position.y().to_string(),
                ],
            )
            .await?;
        if lines.is_none() {
            return Err(anyhow!("Unexpected Empty Response"));
        }
        let json = lines.unwrap().pop().unwrap();
        if &json[0..1] == "{" {
            Ok(serde_json::from_str(json.as_str())?)
        } else {
            Err(anyhow!("{}", json))
        }
    }

    pub async fn cheat_blueprint(
        &self,
        player_id: u32,
        blueprint: String,
        position: &Position,
        direction: u8,
        force_build: bool,
    ) -> anyhow::Result<Vec<FactorioEntity>> {
        let lines = self
            .remote_call(
                "cheat_blueprint",
                vec![
                    &player_id.to_string(),
                    &str_to_lua(&blueprint),
                    &position.x().to_string(),
                    &position.y().to_string(),
                    &direction.to_string(),
                    &force_build.to_string(),
                ],
            )
            .await?;
        if lines.is_none() {
            return Err(anyhow!("Unexpected Empty Response"));
        }
        let mut json = lines.unwrap().pop().unwrap();
        // empty objects/arrays are the same in lua
        if json == "{}" {
            json = String::from("[]");
        }
        Ok(serde_json::from_str(json.as_str())?)
    }

    pub async fn store_map_data(&self, key: &str, value: Value) -> anyhow::Result<()> {
        self.remote_call(
            "store_map_data",
            vec![&str_to_lua(key), &value_to_lua(&value)],
        )
        .await?;
        Ok(())
    }

    pub async fn retrieve_map_data(&self, key: &str) -> anyhow::Result<Option<Value>> {
        let lines = self
            .remote_call("retrieve_map_data", vec![&str_to_lua(key)])
            .await?;
        if lines.is_none() {
            return Ok(None);
        }
        let mut json = lines.unwrap().pop().unwrap();
        // empty objects/arrays are the same in lua
        if json == "{}" {
            json = String::from("[]");
        }
        Ok(Some(serde_json::from_str(json.as_str())?))
    }

    pub async fn move_player(
        &self,
        world: &State<'_, Arc<FactorioWorld>>,
        player_id: u32,
        goal: &Position,
        radius: Option<f64>,
    ) -> anyhow::Result<()> {
        let mut next_action_id = world.as_ref().next_action_id.lock().await;
        let action_id: u32 = *next_action_id;
        *next_action_id = (*next_action_id + 1) % 1000;
        drop(next_action_id);

        let waypoints = self.player_path(&world, player_id, &goal, radius).await?;

        self.action_start_walk_waypoints(action_id, player_id, waypoints)
            .await?;
        loop {
            let (result_id, _) = world.as_ref().rx_actions.recv().unwrap();
            if action_id == result_id {
                // info!("action result: <bright-blue>{}</>", result);
                return Ok(());
            }
        }
    }

    pub async fn player_mine(
        &self,
        world: &State<'_, Arc<FactorioWorld>>,
        player_id: u32,
        name: &str,
        position: &Position,
        count: u32,
    ) -> anyhow::Result<()> {
        let mut next_action_id = world.as_ref().next_action_id.lock().await;
        let action_id: u32 = *next_action_id;
        *next_action_id = (*next_action_id + 1) % 1000;
        drop(next_action_id);
        let player = world.players.get_one(&player_id).unwrap();
        let resource_reach_distance = player.resource_reach_distance.unwrap_or(3) as f64;
        let distance = calculate_distance(&player.position, &position);
        drop(player); // wow, without this factorio (?) freezes (!)
        if distance > resource_reach_distance {
            warn!("too far away, moving first!");
            self.move_player(world, player_id, position, Some(resource_reach_distance))
                .await?;
        }
        self.action_start_mining(action_id, player_id, name, position, count)
            .await?;
        loop {
            let (result_id, _) = world.as_ref().rx_actions.recv().unwrap();
            if action_id == result_id {
                break;
            }
        }
        Ok(())
    }

    pub async fn player_craft(
        &self,
        world: &State<'_, Arc<FactorioWorld>>,
        player_id: u32,
        recipe: &str,
        count: u32,
    ) -> anyhow::Result<()> {
        let mut next_action_id = world.as_ref().next_action_id.lock().await;
        let action_id: u32 = *next_action_id;
        *next_action_id = (*next_action_id + 1) % 1000;
        drop(next_action_id);
        self.action_start_crafting(action_id, player_id, recipe, count)
            .await?;
        loop {
            let (result_id, _) = world.as_ref().rx_actions.recv().unwrap();
            if action_id == result_id {
                break;
            }
        }
        Ok(())
    }

    pub async fn inventory_contents_at(
        &self,
        entities: Vec<RequestEntity>,
    ) -> anyhow::Result<Vec<Option<InventoryResponse>>> {
        let positions: Vec<String> = entities
            .into_iter()
            .map(|entity| {
                let mut map: HashMap<String, String> = HashMap::new();
                map.insert(String::from("name"), str_to_lua(&entity.name));
                map.insert(
                    String::from("position"),
                    vec_to_lua(vec![
                        entity.position.x.to_string(),
                        entity.position.y.to_string(),
                    ]),
                );
                hashmap_to_lua(map)
            })
            .collect();

        let lines = self
            .remote_call("inventory_contents_at", vec![&vec_to_lua(positions)])
            .await?;
        if lines.is_none() {
            return Err(anyhow!("Unexpected Empty Response"));
        }
        let mut json = lines.unwrap().pop().unwrap();
        // empty objects/arrays are the same in lua
        if json == "{}" {
            json = String::from("[]");
        }
        Ok(serde_json::from_str(json.as_str())?)
    }

    pub async fn player_force(&self) -> anyhow::Result<FactorioForce> {
        let lines = self.remote_call("player_force", vec![]).await?;
        if lines.is_none() {
            return Err(anyhow!("Unexpected Empty Response"));
        }
        let json = lines.unwrap().pop().unwrap();
        Ok(serde_json::from_str(json.as_str())?)
    }

    pub async fn place_entity(
        &self,
        player_id: u32,
        item_name: String,
        entity_position: Position,
        direction: u8,
        world: &State<'_, Arc<FactorioWorld>>,
    ) -> anyhow::Result<FactorioEntity> {
        let player = world.players.get_one(&player_id).unwrap();
        let player_position = player.position.clone();
        let build_distance = player.build_distance.unwrap_or(10) as f64;
        drop(player); // wow, without this factorio (?) freezes (!)
        let distance = calculate_distance(&player_position, &entity_position);
        if distance > build_distance {
            warn!("too far away, moving first!");
            self.move_player(world, player_id, &entity_position, Some(build_distance))
                .await?;
        }
        let lines = self
            .remote_call(
                "place_entity",
                vec![
                    &player_id.to_string(),
                    &str_to_lua(&item_name),
                    &position_to_lua(&entity_position),
                    &direction.to_string(),
                ],
            )
            .await?;
        if let Some(lines) = lines {
            if lines.len() != 1 {
                Err(anyhow!("unexpected output {:?}", lines))
            } else {
                let line = &lines[0];
                let chars =
                    UnicodeSegmentation::graphemes(line.as_str(), true).collect::<Vec<&str>>();
                if chars[0] == "{" {
                    Ok(serde_json::from_str(&line).unwrap())
                } else if &line[..] == "§player_blocks_placement§" {
                    for test_direction in 0..8u8 {
                        let test_position = move_position(
                            &player_position,
                            Direction::from_u8(test_direction).unwrap(),
                            5.0,
                        );
                        if self
                            .is_empty(None, Some(test_position.clone()), Some(2.0))
                            .await?
                        {
                            self.move_player(world, player_id, &test_position, Some(1.0))
                                .await?;
                            let lines = self
                                .remote_call(
                                    "place_entity",
                                    vec![
                                        &player_id.to_string(),
                                        &str_to_lua(&item_name),
                                        &position_to_lua(&entity_position),
                                        &direction.to_string(),
                                    ],
                                )
                                .await?;
                            return if let Some(lines) = lines {
                                if lines.len() != 1 {
                                    return Err(anyhow!("unexpected output {:?}", lines));
                                }
                                let line = &lines[0];
                                let chars = UnicodeSegmentation::graphemes(line.as_str(), true)
                                    .collect::<Vec<&str>>();
                                if chars[0] == "{" {
                                    Ok(serde_json::from_str(&line).unwrap())
                                } else if &line[..] == "§player_blocks_placement§" {
                                    Err(anyhow!("player still blocks placement"))
                                } else {
                                    Err(anyhow!("{}", line))
                                }
                            } else {
                                Err(anyhow!("unexpected empty output"))
                            };
                        }
                    }
                    Err(anyhow!(
                        "Error: player blocks placement in all directions! "
                    ))
                } else {
                    Err(anyhow!("{}", line))
                }
            }
        } else {
            Err(anyhow!("unexpected empty output"))
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn insert_to_inventory(
        &self,
        player_id: u32,
        entity_name: String,
        entity_position: Position,
        inventory_type: u32,
        item_name: String,
        item_count: u32,
        world: &State<'_, Arc<FactorioWorld>>,
    ) -> anyhow::Result<()> {
        let player = world.players.get_one(&player_id).unwrap();
        let reach_distance = player.reach_distance.unwrap_or(10) as f64;
        let distance = calculate_distance(&player.position, &entity_position);
        drop(player); // wow, without this factorio (?) freezes (!)
        if distance > reach_distance {
            warn!("too far away, moving first!");
            self.move_player(world, player_id, &entity_position, Some(reach_distance))
                .await?;
        }

        let player_id = player_id.to_string();
        let mut items: HashMap<String, String> = HashMap::new();
        items.insert(String::from("name"), str_to_lua(&item_name));
        items.insert(String::from("count"), item_count.to_string());
        let lines = self
            .remote_call(
                "insert_to_inventory",
                vec![
                    &player_id,
                    &str_to_lua(&entity_name),
                    &position_to_lua(&entity_position),
                    &inventory_type.to_string(),
                    &hashmap_to_lua(items),
                ],
            )
            .await?;
        if lines.is_some() {
            return Err(anyhow!("{:?}", lines.unwrap()));
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn remove_from_inventory(
        &self,
        player_id: u32,
        entity_name: String,
        entity_position: Position,
        inventory_type: u32,
        item_name: String,
        item_count: u32,
        world: &State<'_, Arc<FactorioWorld>>,
    ) -> anyhow::Result<()> {
        let player = world.players.get_one(&player_id).unwrap();
        let reach_distance = player.reach_distance.unwrap_or(10) as f64;
        let distance = calculate_distance(&player.position, &entity_position);
        drop(player); // wow, without this factorio (?) freezes (!)
        if distance > reach_distance {
            warn!("too far away, moving first!");
            self.move_player(world, player_id, &entity_position, Some(reach_distance))
                .await?;
        }
        let player_id = player_id.to_string();
        let mut items: HashMap<String, String> = HashMap::new();
        items.insert(String::from("name"), str_to_lua(&item_name));
        items.insert(String::from("count"), item_count.to_string());
        let lines = self
            .remote_call(
                "remove_from_inventory",
                vec![
                    &player_id,
                    &str_to_lua(&entity_name),
                    &position_to_lua(&entity_position),
                    &inventory_type.to_string(),
                    &hashmap_to_lua(items),
                ],
            )
            .await?;
        if lines.is_some() {
            return Err(anyhow!("{:?}", lines.unwrap()));
        }
        Ok(())
    }

    pub async fn is_empty(
        &self,
        area: Option<Rect>,
        position: Option<Position>,
        radius: Option<f64>,
    ) -> anyhow::Result<bool> {
        let entities = self
            .find_entities_filtered(area.clone(), position.clone(), radius, None, None)
            .await?;
        if entities.len() > 0 {
            return Ok(false);
        }
        let tiles = self
            .find_tiles_filtered(area, position, radius, None)
            .await?;
        for tile in tiles {
            if tile.player_collidable {
                return Ok(false);
            }
        }
        Ok(true)
    }

    // https://lua-api.factorio.com/latest/LuaSurface.html#LuaSurface.find_entities_filtered
    /*
       Table with the following fields:
       area :: BoundingBox (optional)
       position :: Position (optional)
       radius :: double (optional): If given with position, will return all entities within the radius of the position.
       name :: string or array of string (optional)
       type :: string or array of string (optional)
       ghost_name :: string or array of string (optional)
       ghost_type :: string or array of string (optional)
       direction :: defines.direction or array of defines.direction (optional)
       collision_mask :: CollisionMaskLayer or array of CollisionMaskLayer (optional)
       force :: ForceSpecification or array of ForceSpecification (optional)
       to_be_upgraded :: boolean (optional)
       limit :: uint (optional)
       invert :: boolean (optional): If the filters should be inverted. These filters are: name, type, ghost_name, ghost_type, direction, collision_mask, force.
    */
    pub async fn find_entities_filtered(
        &self,
        area: Option<Rect>,
        position: Option<Position>,
        radius: Option<f64>,
        name: Option<String>,
        entity_type: Option<String>,
    ) -> anyhow::Result<Vec<FactorioEntity>> {
        if area.is_none() && (position.is_none() || radius.is_none()) {
            return Err(anyhow!("area or position+radius needed"));
        }
        let mut args: HashMap<String, String> = HashMap::new();
        if let Some(area) = area {
            args.insert(String::from("area"), rect_to_lua(&area));
        }
        if let Some(position) = position {
            args.insert(String::from("position"), position_to_lua(&position));
        }
        if let Some(radius) = radius {
            if radius > 1000.0 {
                return Err(anyhow!("max radius: 1000"));
            }
            args.insert(String::from("radius"), radius.to_string());
        }
        if let Some(name) = name {
            args.insert(String::from("name"), str_to_lua(&name));
        }
        if let Some(entity_type) = entity_type {
            args.insert(String::from("type"), str_to_lua(&entity_type));
        }

        let result = self
            .remote_call(
                "find_entities_filtered",
                vec![hashmap_to_lua(args).as_str()],
            )
            .await?;
        if result.is_none() {
            return Err(anyhow!("Expected result from find_entitites_filtered"));
        }
        let mut json = result.unwrap().pop().unwrap();
        // empty objects/arrays are the same in lua
        if json == "{}" {
            json = String::from("[]");
        }
        Ok(serde_json::from_str(json.as_str())?)
    }

    pub async fn parse_map_exchange_string(
        &self,
        name: &str,
        map_exchange_string: &str,
    ) -> anyhow::Result<()> {
        let result = self
            .remote_call(
                "parse_map_exchange_string",
                vec![&str_to_lua(name), &str_to_lua(map_exchange_string)],
            )
            .await?;
        if result.is_some() {
            return Err(anyhow!("{}", result.unwrap().join("\n")));
        }
        Ok(())
    }
    pub async fn find_tiles_filtered(
        &self,
        area: Option<Rect>,
        position: Option<Position>,
        radius: Option<f64>,
        name: Option<String>,
    ) -> anyhow::Result<Vec<FactorioTile>> {
        if area.is_none() && (position.is_none() || radius.is_none()) {
            return Err(anyhow!("area or position+radius needed"));
        }
        let mut args: HashMap<String, String> = HashMap::new();
        if let Some(area) = area {
            args.insert(String::from("area"), rect_to_lua(&area));
        }
        if let Some(position) = position {
            args.insert(String::from("position"), position_to_lua(&position));
        }
        if let Some(radius) = radius {
            if radius > 1000.0 {
                return Err(anyhow!("max radius: 1000"));
            }
            args.insert(String::from("radius"), radius.to_string());
        }
        if let Some(name) = name {
            args.insert(String::from("name"), str_to_lua(&name));
        }
        let result = self
            .remote_call("find_tiles_filtered", vec![hashmap_to_lua(args).as_str()])
            .await?;
        if result.is_none() {
            return Err(anyhow!("Expected result from find_tiles_filtered"));
        }
        let mut json = result.unwrap().pop().unwrap();
        // empty objects/arrays are the same in lua
        if json == "{}" {
            json = String::from("[]");
        }
        Ok(serde_json::from_str(json.as_str())?)
    }

    async fn async_request_player_path(
        &self,
        player_id: u32,
        goal: &Position,
        radius: Option<f64>,
    ) -> anyhow::Result<u32> {
        let radius = match radius {
            Some(radius) => radius.to_string(),
            None => String::from("nil"),
        };
        let result = self
            .remote_call(
                "async_request_player_path",
                vec![&player_id.to_string(), &position_to_lua(&goal), &radius],
            )
            .await?;
        if result.is_none() {
            return Err(anyhow!("Expected result from request_path"));
        }
        let result = result.unwrap().pop().unwrap();
        match result.parse() {
            Ok(result) => Ok(result),
            Err(_) => Err(anyhow!("{}", result)),
        }
    }

    // https://lua-api.factorio.com/latest/LuaSurface.html#LuaSurface.request_path
    /*
       bounding_box :: BoundingBox
       collision_mask :: CollisionMask or array of string
       start :: Position
       goal :: Position
       force :: LuaForce or string
       radius :: double (optional): How close we need to get to the goal. Default 1.
       pathfind_flags :: PathFindFlags (optional): Flags to affect the pathfinder.
       can_open_gates :: boolean (optional): If the path request can open gates. Default false.
       path_resolution_modifier :: int (optional): The resolution modifier of the pathing. Defaults to 0.
       entity_to_ignore :: LuaEntity (optional): If given, the pathfind will ignore collisions with this entity.
    */
    pub async fn player_path(
        &self,
        world: &State<'_, Arc<FactorioWorld>>,
        player_id: u32,
        goal: &Position,
        radius: Option<f64>,
    ) -> anyhow::Result<Vec<Position>> {
        let id = self
            .async_request_player_path(player_id, goal, radius)
            .await?;
        loop {
            let (result_id, mut result) = world.as_ref().rx_path_requests.recv().unwrap();
            if id == result_id {
                if result == "{}" {
                    result = String::from("[]");
                }
                return Ok(serde_json::from_str(result.as_str())?);
            }
        }
    }

    pub async fn action_start_walk_waypoints(
        &self,
        action_id: u32,
        player_id: u32,
        waypoints: Vec<Position>,
    ) -> anyhow::Result<()> {
        // set_waypoints(action_id, player_id, waypoints)
        let action_id = action_id.to_string();
        let player_id = player_id.to_string();
        let waypoints = waypoints
            .iter()
            .map(|position| position_to_lua(position))
            .collect::<Vec<String>>()
            .join(", ");
        let result = self
            .remote_call(
                "action_start_walk_waypoints",
                vec![&action_id, &player_id, &format!("{{ {} }}", waypoints)],
            )
            .await?;
        if let Some(result) = result {
            return Err(anyhow!("{}", result.join("\n")));
        }
        Ok(())
    }

    pub async fn action_start_mining(
        &self,
        action_id: u32,
        player_id: u32,
        name: &str,
        position: &Position,
        count: u32,
    ) -> anyhow::Result<()> {
        let action_id = action_id.to_string();
        let player_id = player_id.to_string();
        let result = self
            .remote_call(
                "action_start_mining",
                vec![
                    &action_id,
                    &player_id,
                    &str_to_lua(name),
                    &position_to_lua(&position),
                    &count.to_string(),
                ],
            )
            .await?;
        if result.is_some() {
            return Err(anyhow!("{:?}", result.unwrap()));
        }
        Ok(())
    }

    pub async fn action_start_crafting(
        &self,
        action_id: u32,
        player_id: u32,
        recipe: &str,
        count: u32,
    ) -> anyhow::Result<()> {
        let action_id = action_id.to_string();
        let player_id = player_id.to_string();
        let result = self
            .remote_call(
                "action_start_crafting",
                vec![
                    &action_id,
                    &player_id,
                    &str_to_lua(recipe),
                    &count.to_string(),
                ],
            )
            .await?;
        if result.is_some() {
            return Err(anyhow!("{:?}", result.unwrap()));
        }
        Ok(())
    }
}
