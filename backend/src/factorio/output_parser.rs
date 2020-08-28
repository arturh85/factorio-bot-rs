use crate::factorio::ws::{
    FactorioWebSocketServer, PlayerChangedMainInventoryMessage, PlayerChangedPositionMessage,
    PlayerDistanceChangedMessage, PlayerLeftMessage, ResearchCompletedMessage,
};
use crate::types::{
    ChunkObject, ChunkPosition, ChunkResource, FactorioChunk, FactorioEntityPrototype,
    FactorioGraphic, FactorioItemPrototype, FactorioPlayer, FactorioRecipe, FactorioTile,
    PlayerChangedDistanceEvent, PlayerChangedMainInventoryEvent, PlayerChangedPositionEvent,
    Position, Rect,
};
use actix::Addr;
use async_std::sync::Mutex;
use evmap::{ReadHandle, WriteHandle};
use image::RgbaImage;
use num_traits::cast::ToPrimitive;
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

#[derive(Debug)]
pub struct FactorioWorld {
    pub players: ReadHandle<u32, FactorioPlayer>,
    pub chunks: ReadHandle<ChunkPosition, FactorioChunk>,
    pub graphics: ReadHandle<String, FactorioGraphic>,
    pub recipes: ReadHandle<String, FactorioRecipe>,
    pub entity_prototypes: ReadHandle<String, FactorioEntityPrototype>,
    pub item_prototypes: ReadHandle<String, FactorioItemPrototype>,
    pub image_cache: ReadHandle<String, Box<RgbaImage>>,
    pub image_cache_writer: std::sync::Mutex<WriteHandle<String, Box<RgbaImage>>>,
    pub actions: Mutex<HashMap<u32, String>>,
    pub path_requests: Mutex<HashMap<u32, String>>,
    pub next_action_id: Mutex<u32>,
}

unsafe impl Send for FactorioWorld {}
unsafe impl Sync for FactorioWorld {}

pub struct OutputParser {
    world: Arc<FactorioWorld>,
    websocket_server: Option<Addr<FactorioWebSocketServer>>,
    chunks_writer: WriteHandle<ChunkPosition, FactorioChunk>,
    graphics_writer: WriteHandle<String, FactorioGraphic>,
    recipes_writer: WriteHandle<String, FactorioRecipe>,
    entity_prototypes_writer: WriteHandle<String, FactorioEntityPrototype>,
    item_prototypes_writer: WriteHandle<String, FactorioItemPrototype>,
    players_writer: WriteHandle<u32, FactorioPlayer>,
}

impl OutputParser {
    pub async fn parse(&mut self, _tick: u64, action: &str, rest: &str) -> anyhow::Result<()> {
        match action {
            "objects" => {
                let colon_pos = rest.find(':').unwrap();
                let rect: Rect = rest[0..colon_pos].parse()?;
                let chunk_position = ChunkPosition {
                    x: (*rect.left_top.x).to_i32().unwrap() / 32,
                    y: (*rect.left_top.y).to_i32().unwrap() / 32,
                };
                let mut objects = &rest[colon_pos + 1..];
                if objects == "{}" {
                    objects = "[]"
                }
                // if chunk_position.x.abs() < 2 && chunk_position.y.abs() < 2 {
                //     info!(
                //         "objects for {} / {}: {}",
                //         chunk_position.x, chunk_position.y, objects
                //     );
                // }
                let objects: Vec<ChunkObject> = serde_json::from_str(objects).unwrap();
                if self.chunks_writer.contains_key(&chunk_position) {
                    let existing_chunk = self.chunks_writer.get_one(&chunk_position).unwrap(); // unwrap OK because of contains_key
                    let chunk = FactorioChunk {
                        objects,
                        resources: existing_chunk.resources.to_vec(),
                        tiles: existing_chunk.tiles.clone(),
                    };
                    drop(existing_chunk);
                    self.chunks_writer.empty(chunk_position.clone());
                    self.chunks_writer.insert(chunk_position, chunk);
                } else {
                    self.chunks_writer.insert(
                        chunk_position,
                        FactorioChunk {
                            objects,
                            resources: vec![],
                            tiles: vec![],
                        },
                    );
                }
                self.chunks_writer.refresh();
            }
            "resources" => {
                // info!("tick!");
                // 123807 resources 576,-384;608,-352:
                let parts: Vec<&str> = rest.split(':').collect();
                let rect: Rect = parts[0].parse()?;
                let chunk_position = ChunkPosition {
                    x: (*rect.left_top.x).to_i32().unwrap() / 32,
                    y: (*rect.left_top.y).to_i32().unwrap() / 32,
                };
                let resources = parts[1];
                let resources = if resources.len() > 1 {
                    &resources[2..]
                } else {
                    &""
                };

                let resources = if resources.is_empty() {
                    vec![]
                } else {
                    resources
                        .split(',')
                        .into_iter()
                        .map(|resource_string| {
                            let resource_parts: Vec<&str> = resource_string.split(' ').collect();
                            ChunkResource {
                                name: resource_parts[0].into(),
                                position: format!("{},{}", resource_parts[1], resource_parts[2])
                                    .parse()
                                    .unwrap(),
                            }
                        })
                        .collect()
                };
                if self.chunks_writer.contains_key(&chunk_position) {
                    let existing_chunk = self.chunks_writer.get_one(&chunk_position).unwrap(); // unwrap OK
                    let chunk = FactorioChunk {
                        resources,
                        objects: existing_chunk.objects.clone(),
                        tiles: existing_chunk.tiles.clone(),
                    };
                    drop(existing_chunk);
                    self.chunks_writer.empty(chunk_position.clone());
                    self.chunks_writer.insert(chunk_position, chunk);
                } else {
                    self.chunks_writer.insert(
                        chunk_position,
                        FactorioChunk {
                            resources,
                            objects: vec![],
                            tiles: vec![],
                        },
                    );
                }
                self.chunks_writer.refresh();
            }
            "graphics" => {
                // 0 graphics: spark-explosion*__core__/graphics/empty.png:1:1:0:0:0:0:1|spark-explosion-higher*__core__/graphics/empty.png:1:1:0:0:0:0:1|
                let graphics: Vec<&str> = rest.split('|').collect();
                for graphic in graphics {
                    let parts: Vec<&str> = graphic.split(':').collect();
                    let parts2: Vec<&str> = parts[0].split('*').collect();
                    let name: String = parts2[0].into();
                    let name2 = name.clone();
                    let _graphics = FactorioGraphic {
                        entity_name: name,
                        image_path: parts2[1].into(),
                        width: parts[1].parse().unwrap(),
                        height: parts[1].parse().unwrap(),
                    };
                    self.graphics_writer.insert(name2, _graphics);
                }
                self.graphics_writer.refresh();
            }
            "entity_prototypes" => {
                for entity_prototype in rest.split('$') {
                    let entity_prototype: FactorioEntityPrototype =
                        serde_json::from_str(entity_prototype)?;
                    self.entity_prototypes_writer
                        .insert(entity_prototype.name.clone(), entity_prototype);
                }
                self.entity_prototypes_writer.refresh();
            }
            "item_prototypes" => {
                for item_prototype in rest.split('$') {
                    let item_prototype: FactorioItemPrototype =
                        serde_json::from_str(item_prototype)?;
                    self.item_prototypes_writer
                        .insert(item_prototype.name.clone(), item_prototype);
                }
                self.item_prototypes_writer.refresh();
            }
            "recipes" => {
                // info!("recipes found: {}", rest);
                for recipe in rest.split('$') {
                    let recipe: FactorioRecipe = serde_json::from_str(recipe)?;
                    self.recipes_writer.insert(recipe.name.clone(), recipe);
                }
                self.recipes_writer.refresh();
            }
            "action_completed" => {
                if let Some(pos) = rest.find(' ') {
                    let action_status = &rest[0..pos];
                    let rest = &rest[pos + 1..];
                    let action_id: u32 = match rest.find(' ') {
                        Some(pos) => (&rest[0..pos]).parse()?,
                        None => rest.parse()?,
                    };
                    let result = match action_status {
                        "ok" => "ok",
                        "fail" => {
                            let pos = rest.find(' ').unwrap();
                            &rest[pos + 1..]
                        }
                        _ => panic!(format!("unexpected action_completed: {}", action_status)),
                    };
                    let mut actions = self.world.actions.lock().await;
                    actions.insert(action_id, String::from(result));
                }
            }
            "on_script_path_request_finished" => {
                let parts: Vec<&str> = rest.split('#').collect();
                let id: u32 = parts[0].parse()?;
                let mut path_requests = self.world.path_requests.lock().await;
                path_requests.insert(id, String::from(parts[1]));
                // info!("XXX player_path XXX sending");
                // self.tx_path_requests
                //     .send((id, String::from(parts[1])))
                //     .unwrap();
                // info!("XXX player_path XXX sended");
            }
            "STATIC_DATA_END" => {
                // info!("STATIC_DATA_END!");
            }
            "on_player_left_game" => {
                let player_id: u32 = rest.parse()?;
                self.players_writer.empty(player_id);
                self.players_writer.refresh();
                if let Some(websocket_server) = self.websocket_server.as_ref() {
                    websocket_server
                        .send(PlayerLeftMessage { player_id })
                        .await?;
                }
            }
            "on_research_finished" => {
                if let Some(websocket_server) = self.websocket_server.as_ref() {
                    websocket_server.send(ResearchCompletedMessage {}).await?;
                }
            }
            "on_player_main_inventory_changed" => {
                let event: PlayerChangedMainInventoryEvent = serde_json::from_str(rest)?;
                if self.players_writer.contains_key(&event.player_id) {
                    let existing_player = self.players_writer.get_one(&event.player_id).unwrap();
                    let player = FactorioPlayer {
                        player_id: event.player_id,
                        position: existing_player.position.clone(),
                        main_inventory: event.main_inventory,
                        build_distance: existing_player.build_distance,
                        reach_distance: existing_player.reach_distance,
                        drop_item_distance: existing_player.drop_item_distance,
                        item_pickup_distance: existing_player.item_pickup_distance,
                        loot_pickup_distance: existing_player.loot_pickup_distance,
                        resource_reach_distance: existing_player.resource_reach_distance,
                    };
                    if let Some(websocket_server) = self.websocket_server.as_ref() {
                        websocket_server
                            .send(PlayerChangedMainInventoryMessage {
                                player: player.clone(),
                            })
                            .await?;
                    }
                    drop(existing_player);
                    self.players_writer.empty(event.player_id);
                    self.players_writer.insert(event.player_id, player);
                } else {
                    let player = FactorioPlayer {
                        player_id: event.player_id,
                        position: Position::new(0.0, 0.0),
                        main_inventory: event.main_inventory.clone(),
                        build_distance: 0,
                        reach_distance: 0,
                        drop_item_distance: 0,
                        item_pickup_distance: 0,
                        loot_pickup_distance: 0,
                        resource_reach_distance: 0,
                    };
                    if let Some(websocket_server) = self.websocket_server.as_ref() {
                        websocket_server
                            .send(PlayerChangedMainInventoryMessage {
                                player: player.clone(),
                            })
                            .await?;
                    }
                    self.players_writer.insert(event.player_id, player);
                }
                self.players_writer.refresh();
            }
            "on_player_changed_position" => {
                let event: PlayerChangedPositionEvent = serde_json::from_str(rest)?;
                if self.players_writer.contains_key(&event.player_id) {
                    let existing_player = self.players_writer.get_one(&event.player_id).unwrap();
                    let player = FactorioPlayer {
                        player_id: event.player_id,
                        position: event.position,
                        main_inventory: existing_player.main_inventory.clone(),
                        build_distance: existing_player.build_distance,
                        reach_distance: existing_player.reach_distance,
                        drop_item_distance: existing_player.drop_item_distance,
                        item_pickup_distance: existing_player.item_pickup_distance,
                        loot_pickup_distance: existing_player.loot_pickup_distance,
                        resource_reach_distance: existing_player.resource_reach_distance,
                    };
                    drop(existing_player);
                    if let Some(websocket_server) = self.websocket_server.as_ref() {
                        websocket_server
                            .send(PlayerChangedPositionMessage {
                                player: player.clone(),
                            })
                            .await?;
                    }
                    self.players_writer.empty(event.player_id);
                    self.players_writer.insert(event.player_id, player);
                } else {
                    let player = FactorioPlayer {
                        player_id: event.player_id,
                        position: event.position,
                        main_inventory: Box::new(BTreeMap::new()),
                        build_distance: 0,
                        reach_distance: 0,
                        drop_item_distance: 0,
                        item_pickup_distance: 0,
                        loot_pickup_distance: 0,
                        resource_reach_distance: 0,
                    };
                    if let Some(websocket_server) = self.websocket_server.as_ref() {
                        websocket_server
                            .send(PlayerChangedPositionMessage {
                                player: player.clone(),
                            })
                            .await?;
                    }
                    self.players_writer.insert(event.player_id, player);
                }
                self.players_writer.refresh();
            }
            "on_player_changed_distance" => {
                let event: PlayerChangedDistanceEvent = serde_json::from_str(rest)?;
                if self.players_writer.contains_key(&event.player_id) {
                    let existing_player = self.players_writer.get_one(&event.player_id).unwrap();
                    let player = FactorioPlayer {
                        player_id: event.player_id,
                        position: existing_player.position.clone(),
                        main_inventory: existing_player.main_inventory.clone(),
                        build_distance: event.build_distance,
                        reach_distance: event.reach_distance,
                        drop_item_distance: event.drop_item_distance,
                        item_pickup_distance: event.item_pickup_distance,
                        loot_pickup_distance: event.loot_pickup_distance,
                        resource_reach_distance: event.resource_reach_distance,
                    };
                    drop(existing_player);
                    if let Some(websocket_server) = self.websocket_server.as_ref() {
                        websocket_server
                            .send(PlayerDistanceChangedMessage {
                                player: player.clone(),
                            })
                            .await?;
                    }
                    self.players_writer.empty(event.player_id);
                    self.players_writer.insert(event.player_id, player);
                } else {
                    let player = FactorioPlayer {
                        player_id: event.player_id,
                        position: Position::new(0.0, 0.0),
                        main_inventory: Box::new(BTreeMap::new()),
                        build_distance: event.build_distance,
                        reach_distance: event.reach_distance,
                        drop_item_distance: event.drop_item_distance,
                        item_pickup_distance: event.item_pickup_distance,
                        loot_pickup_distance: event.loot_pickup_distance,
                        resource_reach_distance: event.resource_reach_distance,
                    };
                    if let Some(websocket_server) = self.websocket_server.as_ref() {
                        websocket_server
                            .send(PlayerDistanceChangedMessage {
                                player: player.clone(),
                            })
                            .await?;
                    }
                    self.players_writer.insert(event.player_id, player);
                }
                self.players_writer.refresh();
            }
            "mined_item" => {
                // info!("tick!");
            }
            "tick" => {
                // info!("tick!");
            }
            "tiles" => {
                let colon_pos = rest.find(':').unwrap();
                let rect: Rect = rest[0..colon_pos].parse()?;
                let chunk_position = ChunkPosition {
                    x: (*rect.left_top.x).to_i32().unwrap() / 32,
                    y: (*rect.left_top.y).to_i32().unwrap() / 32,
                };
                let tiles: Vec<FactorioTile> = rest[colon_pos + 1..]
                    .split(',')
                    .enumerate()
                    .map(|(index, tile)| {
                        let parts: Vec<&str> = tile.split(':').collect();
                        FactorioTile {
                            name: parts[0].trim().into(),
                            player_collidable: parts[1].parse::<u8>().unwrap() == 1,
                            position: Position::new(
                                (chunk_position.x * 32 + (index % 32) as i32) as f64,
                                (chunk_position.y * 32 + (index / 32) as i32) as f64,
                            ),
                        }
                    })
                    .collect();
                if self.chunks_writer.contains_key(&chunk_position) {
                    let existing_chunk = self.chunks_writer.get_one(&chunk_position).unwrap(); // unwrap OK because of contains_key
                    let chunk = FactorioChunk {
                        objects: existing_chunk.objects.clone(),
                        resources: existing_chunk.resources.to_vec(),
                        tiles,
                    };
                    drop(existing_chunk);
                    self.chunks_writer.empty(chunk_position.clone());
                    self.chunks_writer.insert(chunk_position, chunk);
                } else {
                    self.chunks_writer.insert(
                        chunk_position,
                        FactorioChunk {
                            objects: vec![],
                            resources: vec![],
                            tiles,
                        },
                    );
                }
                self.chunks_writer.refresh();
            }
            _ => {
                error!("<red>unexpected action</>: <bright-blue>{}</>", action);
            }
        };
        Ok(())
    }

    #[allow(clippy::new_without_default)]
    pub fn new(websocket_server: Option<Addr<FactorioWebSocketServer>>) -> Self {
        let (players_reader, players_writer) = evmap::new::<u32, FactorioPlayer>();
        let (chunks_reader, chunks_writer) = evmap::new::<ChunkPosition, FactorioChunk>();
        let (graphics_reader, graphics_writer) = evmap::new::<String, FactorioGraphic>();
        let (recipes_reader, recipes_writer) = evmap::new::<String, FactorioRecipe>();
        let (image_cache, image_cache_writer) = evmap::new::<String, Box<RgbaImage>>();
        let (entity_prototypes_reader, entity_prototypes_writer) =
            evmap::new::<String, FactorioEntityPrototype>();
        let (item_prototypes_reader, item_prototypes_writer) =
            evmap::new::<String, FactorioItemPrototype>();

        OutputParser {
            websocket_server,
            players_writer,
            chunks_writer,
            graphics_writer,
            recipes_writer,
            entity_prototypes_writer,
            item_prototypes_writer,
            world: Arc::new(FactorioWorld {
                image_cache,
                image_cache_writer: std::sync::Mutex::new(image_cache_writer),
                players: players_reader,
                chunks: chunks_reader,
                graphics: graphics_reader,
                recipes: recipes_reader,
                entity_prototypes: entity_prototypes_reader,
                item_prototypes: item_prototypes_reader,
                actions: Mutex::new(HashMap::default()),
                path_requests: Mutex::new(HashMap::default()),
                next_action_id: Mutex::new(1),
            }),
        }
    }

    pub fn world(&self) -> Arc<FactorioWorld> {
        self.world.clone()
    }
}
