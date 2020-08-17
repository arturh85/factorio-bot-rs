use crate::types::{
    ChunkObject, ChunkPosition, ChunkResource, FactorioChunk, FactorioEntityPrototype,
    FactorioGraphic, FactorioItemPrototype, FactorioPlayer, FactorioRecipe, FactorioTile,
    PlayerChangedPositionEvent, PlayerMainInventoryChangedEvent, Position, Rect,
};
use async_std::sync::Mutex;
use evmap::{ReadHandle, WriteHandle};
use image::RgbaImage;
use noisy_float::types::r64;
use num_traits::cast::ToPrimitive;
use std::collections::{BTreeMap, HashMap};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc};

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
    pub rx_actions: Receiver<(u32, String)>,
    pub rx_path_requests: Receiver<(u32, String)>,
    pub rx_player_inventory_changed: Receiver<u32>,
}

unsafe impl Send for FactorioWorld {}
unsafe impl Sync for FactorioWorld {}

pub struct OutputParser {
    world: Arc<FactorioWorld>,
    chunks_writer: WriteHandle<ChunkPosition, FactorioChunk>,
    graphics_writer: WriteHandle<String, FactorioGraphic>,
    recipes_writer: WriteHandle<String, FactorioRecipe>,
    entity_prototypes_writer: WriteHandle<String, FactorioEntityPrototype>,
    item_prototypes_writer: WriteHandle<String, FactorioItemPrototype>,
    players_writer: WriteHandle<u32, FactorioPlayer>,
    tx_actions: Sender<(u32, String)>,
    tx_path_requests: Sender<(u32, String)>,
    tx_player_inventory_changed: Sender<u32>,
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
                if chunk_position.x.abs() < 2 && chunk_position.y.abs() < 2 {
                    info!(
                        "objects for {} / {}: {}",
                        chunk_position.x, chunk_position.y, objects
                    );
                }
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
                    // let mut actions = self.world.actions.lock().await;
                    // actions.insert(action_id, String::from(result));
                    self.tx_actions
                        .send((action_id, String::from(result)))
                        .unwrap();
                }
            }
            "on_script_path_request_finished" => {
                let parts: Vec<&str> = rest.split('#').collect();
                let id: u32 = parts[0].parse()?;
                // let mut path_requests = self.world.path_requests.lock().await;
                // path_requests.insert(id, String::from(parts[1]));
                // info!("XXX player_path XXX sending");
                self.tx_path_requests
                    .send((id, String::from(parts[1])))
                    .unwrap();
                // info!("XXX player_path XXX sended");
            }
            "STATIC_DATA_END" => {
                // info!("STATIC_DATA_END!");
            }
            "on_player_left_game" => {
                self.players_writer.empty(rest.parse()?);
            }
            "on_player_main_inventory_changed" => {
                let event: PlayerMainInventoryChangedEvent = serde_json::from_str(rest)?;
                if self.players_writer.contains_key(&event.player_id) {
                    let player = FactorioPlayer {
                        player_id: event.player_id,
                        position: self
                            .players_writer
                            .get_one(&event.player_id)
                            .unwrap()
                            .position
                            .clone(),
                        main_inventory: event.main_inventory,
                    };
                    self.players_writer.empty(event.player_id);
                    self.players_writer.insert(event.player_id, player);
                } else {
                    self.players_writer.insert(
                        event.player_id,
                        FactorioPlayer {
                            player_id: event.player_id,
                            position: Position {
                                x: Box::new(r64(0.0)),
                                y: Box::new(r64(0.0)),
                            },
                            main_inventory: event.main_inventory.clone(),
                        },
                    );
                }
                self.players_writer.refresh();
                self.tx_player_inventory_changed.send(event.player_id)?;
            }
            "on_player_changed_position" => {
                let event: PlayerChangedPositionEvent = serde_json::from_str(rest)?;
                if self.players_writer.contains_key(&event.player_id) {
                    let player = FactorioPlayer {
                        player_id: event.player_id,
                        position: event.position,
                        main_inventory: self
                            .players_writer
                            .get_one(&event.player_id)
                            .unwrap()
                            .main_inventory
                            .clone(),
                    };
                    self.players_writer.empty(event.player_id);
                    self.players_writer.insert(event.player_id, player);
                } else {
                    self.players_writer.insert(
                        event.player_id,
                        FactorioPlayer {
                            player_id: event.player_id,
                            position: event.position,
                            main_inventory: Box::new(BTreeMap::new()),
                        },
                    );
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
                            position: Position {
                                x: Box::new(r64(
                                    (chunk_position.x * 32 + (index % 32) as i32) as f64
                                )),
                                y: Box::new(r64(
                                    (chunk_position.y * 32 + (index / 32) as i32) as f64
                                )),
                            },
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
    pub fn new() -> Self {
        let (players_reader, players_writer) = evmap::new::<u32, FactorioPlayer>();
        let (chunks_reader, chunks_writer) = evmap::new::<ChunkPosition, FactorioChunk>();
        let (graphics_reader, graphics_writer) = evmap::new::<String, FactorioGraphic>();
        let (recipes_reader, recipes_writer) = evmap::new::<String, FactorioRecipe>();
        let (image_cache, image_cache_writer) = evmap::new::<String, Box<RgbaImage>>();
        let (entity_prototypes_reader, entity_prototypes_writer) =
            evmap::new::<String, FactorioEntityPrototype>();
        let (item_prototypes_reader, item_prototypes_writer) =
            evmap::new::<String, FactorioItemPrototype>();
        let (tx_actions, rx_actions) = mpsc::channel();
        let (tx_path_requests, rx_path_requests) = mpsc::channel();
        let (tx_player_inventory_changed, rx_player_inventory_changed) = mpsc::channel();

        OutputParser {
            players_writer,
            chunks_writer,
            graphics_writer,
            recipes_writer,
            entity_prototypes_writer,
            item_prototypes_writer,
            tx_actions,
            tx_path_requests,
            tx_player_inventory_changed,

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
                rx_actions,
                rx_path_requests,
                rx_player_inventory_changed,
            }),
        }
    }

    pub fn world(&self) -> Arc<FactorioWorld> {
        self.world.clone()
    }
}
