use crate::factorio::world::{FactorioWorld, FactorioWorldWriter};
use crate::factorio::ws::{
    FactorioWebSocketServer, PlayerChangedMainInventoryMessage, PlayerChangedPositionMessage,
    PlayerDistanceChangedMessage, PlayerLeftMessage, ResearchCompletedMessage,
};
use crate::types::{
    ChunkPosition, FactorioEntity, FactorioEntityPrototype, FactorioGraphic, FactorioItemPrototype,
    FactorioRecipe, FactorioTile, PlayerChangedDistanceEvent, PlayerChangedMainInventoryEvent,
    PlayerChangedPositionEvent, Pos, Position, Rect,
};
use actix::Addr;
use std::sync::Arc;

pub struct OutputParser {
    world: FactorioWorldWriter,
    websocket_server: Option<Addr<FactorioWebSocketServer>>,
}

impl OutputParser {
    pub async fn parse(&mut self, _tick: u64, action: &str, rest: &str) -> anyhow::Result<()> {
        match action {
            "entities" => {
                let colon_pos = rest.find(':').unwrap();
                let rect: Rect = rest[0..colon_pos].parse()?;
                let pos: Pos = (&rect.left_top).into();
                let chunk_position: ChunkPosition = (&pos).into();
                let mut entities = &rest[colon_pos + 1..];
                if entities == "{}" {
                    entities = "[]"
                }
                let entities: Vec<FactorioEntity> = serde_json::from_str(entities).unwrap();
                self.world.update_chunk_entities(chunk_position, entities)?;
            }
            "tiles" => {
                let colon_pos = rest.find(':').unwrap();
                let rect: Rect = rest[0..colon_pos].parse()?;
                let pos: Pos = (&rect.left_top).into();
                let chunk_position: ChunkPosition = (&pos).into();
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
                self.world.update_chunk_tiles(chunk_position, tiles)?;
            }
            "graphics" => {
                // 0 graphics: spark-explosion*__core__/graphics/empty.png:1:1:0:0:0:0:1|spark-explosion-higher*__core__/graphics/empty.png:1:1:0:0:0:0:1|
                let graphics: Vec<FactorioGraphic> = rest
                    .split('|')
                    .map(|graphic| {
                        let parts: Vec<&str> = graphic.split(':').collect();
                        let parts2: Vec<&str> = parts[0].split('*').collect();
                        FactorioGraphic {
                            entity_name: parts2[0].into(),
                            image_path: parts2[1].into(),
                            width: parts[1].parse().unwrap(),
                            height: parts[1].parse().unwrap(),
                        }
                    })
                    .collect();
                self.world.update_graphics(graphics)?;
            }
            "entity_prototypes" => {
                let entity_prototypes: Vec<FactorioEntityPrototype> = rest
                    .split('$')
                    .map(|entity_prototype| {
                        serde_json::from_str(entity_prototype)
                            .expect("failed to deserialize entity prototype")
                    })
                    .collect();
                self.world.update_entity_prototypes(entity_prototypes)?;
            }
            "item_prototypes" => {
                let item_prototypes: Vec<FactorioItemPrototype> = rest
                    .split('$')
                    .map(|item_prototype| {
                        serde_json::from_str(item_prototype)
                            .expect("failed to deserialize item prototype")
                    })
                    .collect();
                self.world.update_item_prototypes(item_prototypes)?;
            }
            "recipes" => {
                let recipes: Vec<FactorioRecipe> = rest
                    .split('$')
                    .map(|recipe| {
                        serde_json::from_str(recipe).expect("failed to deserialize recipe")
                    })
                    .collect();
                self.world.update_recipes(recipes)?;
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
                    let world = self.world.world();
                    let mut actions = world.actions.lock().await;
                    actions.insert(action_id, String::from(result));
                }
            }
            "on_script_path_request_finished" => {
                let parts: Vec<&str> = rest.split('#').collect();
                let id: u32 = parts[0].parse()?;
                let world = self.world.world();
                let mut path_requests = world.path_requests.lock().await;
                path_requests.insert(id, String::from(parts[1]));
                // info!("XXX player_path XXX sending");
                // self.tx_path_requests
                //     .send((id, String::from(parts[1])))
                //     .unwrap();
                // info!("XXX player_path XXX sended");
            }
            "STATIC_DATA_END" => {
                // handled by OutputReader
            }
            "on_player_left_game" => {
                let player_id: u32 = rest.parse()?;
                self.world.remove_player(player_id)?;
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
                let player_id = event.player_id;
                self.world.player_changed_main_inventory(event)?;
                if let Some(websocket_server) = self.websocket_server.as_ref() {
                    websocket_server
                        .send(PlayerChangedMainInventoryMessage {
                            player: self.world().players.get_one(&player_id).unwrap().clone(),
                        })
                        .await?;
                }
            }
            "on_player_changed_position" => {
                let event: PlayerChangedPositionEvent = serde_json::from_str(rest)?;
                let player_id = event.player_id;
                self.world.player_changed_position(event)?;
                if let Some(websocket_server) = self.websocket_server.as_ref() {
                    websocket_server
                        .send(PlayerChangedPositionMessage {
                            player: self.world().players.get_one(&player_id).unwrap().clone(),
                        })
                        .await?;
                }
            }
            "on_player_changed_distance" => {
                let event: PlayerChangedDistanceEvent = serde_json::from_str(rest)?;
                let player_id = event.player_id;
                self.world.player_changed_distance(event)?;
                if let Some(websocket_server) = self.websocket_server.as_ref() {
                    websocket_server
                        .send(PlayerDistanceChangedMessage {
                            player: self.world().players.get_one(&player_id).unwrap().clone(),
                        })
                        .await?;
                }
            }
            "mined_item" => {
                // info!("tick!");
            }
            "tick" => {
                // info!("tick!");
            }
            _ => {
                error!("<red>unexpected action</>: <bright-blue>{}</>", action);
            }
        };
        Ok(())
    }

    #[allow(clippy::new_without_default)]
    pub fn new(websocket_server: Option<Addr<FactorioWebSocketServer>>) -> Self {
        OutputParser {
            websocket_server,
            world: FactorioWorldWriter::new(),
        }
    }

    pub fn world(&self) -> Arc<FactorioWorld> {
        self.world.world()
    }
}
