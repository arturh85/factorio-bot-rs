use crate::factorio::flow::FlowGraph;
use crate::factorio::util::{add_to_rect, rect_fields, rect_floor};
use crate::types::{
    ChunkPosition, FactorioChunk, FactorioEntity, FactorioEntityPrototype, FactorioGraphic,
    FactorioItemPrototype, FactorioPlayer, FactorioRecipe, FactorioTile,
    PlayerChangedDistanceEvent, PlayerChangedMainInventoryEvent, PlayerChangedPositionEvent, Pos,
    Position,
};
use async_std::sync::Mutex;
use evmap::{ReadHandle, WriteHandle};
use image::RgbaImage;
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

#[derive(EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum EntityName {
    Stone,
    Coal,
    IronOre,
}

#[derive(Debug)]
pub struct FactorioWorld {
    pub players: ReadHandle<u32, FactorioPlayer>,
    pub chunks: ReadHandle<ChunkPosition, FactorioChunk>,
    pub blocked: ReadHandle<Pos, bool>,
    pub resources: ReadHandle<String, Vec<Position>>,
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

pub struct FactorioWorldWriter {
    pub world: Arc<FactorioWorld>,
    chunks_writer: WriteHandle<ChunkPosition, FactorioChunk>,
    graphics_writer: WriteHandle<String, FactorioGraphic>,
    recipes_writer: WriteHandle<String, FactorioRecipe>,
    entity_prototypes_writer: WriteHandle<String, FactorioEntityPrototype>,
    item_prototypes_writer: WriteHandle<String, FactorioItemPrototype>,
    players_writer: WriteHandle<u32, FactorioPlayer>,
    blocked_writer: WriteHandle<Pos, bool>,
    resources_writer: WriteHandle<String, Vec<Position>>,
    flow: Arc<Mutex<FlowGraph>>,
}

impl FactorioWorldWriter {
    pub fn update_entity_prototypes(
        &mut self,
        entity_prototypes: Vec<FactorioEntityPrototype>,
    ) -> anyhow::Result<()> {
        for entity_prototype in entity_prototypes {
            self.entity_prototypes_writer
                .insert(entity_prototype.name.clone(), entity_prototype);
        }
        self.entity_prototypes_writer.refresh();
        Ok(())
    }

    pub fn update_item_prototypes(
        &mut self,
        item_prototypes: Vec<FactorioItemPrototype>,
    ) -> anyhow::Result<()> {
        for item_prototype in item_prototypes {
            self.item_prototypes_writer
                .insert(item_prototype.name.clone(), item_prototype);
        }
        self.item_prototypes_writer.refresh();
        Ok(())
    }
    pub fn remove_player(&mut self, player_id: u32) -> anyhow::Result<()> {
        self.players_writer.empty(player_id);
        self.players_writer.refresh();
        Ok(())
    }

    pub fn player_changed_distance(
        &mut self,
        event: PlayerChangedDistanceEvent,
    ) -> anyhow::Result<()> {
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
            self.players_writer.insert(event.player_id, player);
        }
        self.players_writer.refresh();
        Ok(())
    }

    pub fn player_changed_position(
        &mut self,
        event: PlayerChangedPositionEvent,
    ) -> anyhow::Result<()> {
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
            self.players_writer.empty(event.player_id);
            self.players_writer.insert(event.player_id, player);
        } else {
            let player = FactorioPlayer {
                player_id: event.player_id,
                position: event.position,
                ..Default::default()
            };
            self.players_writer.insert(event.player_id, player);
        }
        self.players_writer.refresh();
        Ok(())
    }

    pub fn player_changed_main_inventory(
        &mut self,
        event: PlayerChangedMainInventoryEvent,
    ) -> anyhow::Result<()> {
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
            drop(existing_player);
            self.players_writer.empty(event.player_id);
            self.players_writer.insert(event.player_id, player);
        } else {
            let player = FactorioPlayer {
                player_id: event.player_id,
                main_inventory: event.main_inventory.clone(),
                ..Default::default()
            };
            self.players_writer.insert(event.player_id, player);
        }
        self.players_writer.refresh();
        Ok(())
    }

    pub fn update_recipes(&mut self, recipes: Vec<FactorioRecipe>) -> anyhow::Result<()> {
        for recipe in recipes {
            self.recipes_writer.insert(recipe.name.clone(), recipe);
        }
        self.recipes_writer.refresh();
        Ok(())
    }

    pub fn update_graphics(&mut self, graphics: Vec<FactorioGraphic>) -> anyhow::Result<()> {
        for graphic in graphics {
            self.graphics_writer
                .insert(graphic.entity_name.clone(), graphic);
        }
        self.graphics_writer.refresh();
        Ok(())
    }

    pub fn update_chunk_tiles(
        &mut self,
        chunk_position: ChunkPosition,
        tiles: Vec<FactorioTile>,
    ) -> anyhow::Result<()> {
        for tile in &tiles {
            if tile.player_collidable {
                self.blocked_writer.insert((&tile.position).into(), false);
            }
        }
        self.blocked_writer.refresh();
        if self.chunks_writer.contains_key(&chunk_position) {
            let existing_chunk = self.chunks_writer.get_one(&chunk_position).unwrap(); // unwrap OK because of contains_key
            let chunk = FactorioChunk {
                entities: existing_chunk.entities.clone(),
                tiles,
            };
            drop(existing_chunk);
            self.chunks_writer.empty(chunk_position.clone());
            self.chunks_writer.insert(chunk_position, chunk);
        } else {
            self.chunks_writer.insert(
                chunk_position,
                FactorioChunk {
                    entities: vec![],
                    tiles,
                },
            );
        }
        self.chunks_writer.refresh();
        Ok(())
    }

    #[allow(clippy::map_clone)]
    pub fn update_chunk_entities(
        &mut self,
        chunk_position: ChunkPosition,
        entities: Vec<FactorioEntity>,
    ) -> anyhow::Result<()> {
        // first update blocked entities
        let mut resources: HashMap<String, Vec<Position>> = HashMap::new();
        for entity in &entities {
            // exclude resources like iron-ore which do not block
            match &entity.entity_type[..] {
                "resource" => {
                    // if entity.name == "stone" {
                    //     warn!("stone at {}", entity.position);
                    // }
                    match resources.get_mut(&entity.name) {
                        Some(vec) => {
                            vec.push(entity.position.clone());
                        }
                        None => match self.world.resources.get_one(&entity.name) {
                            Some(vec) => {
                                let mut vec = vec.clone();
                                vec.push(entity.position.clone());
                                resources.insert(entity.name.clone(), vec);
                            }
                            None => {
                                resources
                                    .insert(entity.name.clone(), vec![entity.position.clone()]);
                            }
                        },
                    };
                }
                _ => match self.world.entity_prototypes.get_one(&entity.name) {
                    Some(entity_prototype) => {
                        let collision_box =
                            add_to_rect(&entity_prototype.collision_box, &entity.position);
                        let rect = rect_floor(&collision_box);
                        for position in rect_fields(&rect) {
                            self.blocked_writer
                                .insert((&position).into(), entity.entity_type == "tree");
                        }
                    }
                    None => {
                        self.blocked_writer
                            .insert((&entity.position).into(), entity.entity_type == "tree");
                    }
                },
            }
        }

        if self.chunks_writer.contains_key(&chunk_position) {
            let existing_chunk = self.chunks_writer.get_one(&chunk_position).unwrap(); // unwrap OK because of contains_key
            let chunk = FactorioChunk {
                entities,
                tiles: existing_chunk.tiles.clone(),
            };
            drop(existing_chunk);
            self.chunks_writer.empty(chunk_position.clone());
            self.chunks_writer.insert(chunk_position.clone(), chunk);
        } else {
            self.chunks_writer.insert(
                chunk_position,
                FactorioChunk {
                    entities,
                    tiles: vec![],
                },
            );
        }
        for (k, v) in resources {
            let previous_value = self.resources_writer.get_one(&k).map(|value| value.clone());
            if let Some(previous_value) = previous_value {
                self.resources_writer.remove(k.clone(), previous_value);
            }
            self.resources_writer.insert(k.clone(), v.clone());
        }
        self.resources_writer.refresh();

        self.blocked_writer.refresh();
        self.chunks_writer.refresh();
        Ok(())
    }

    pub fn import(&mut self, world: Arc<FactorioWorld>) -> anyhow::Result<()> {
        if let Some(players) = &world.players.read() {
            for (player_id, player) in players {
                if let Some(player) = player.get_one() {
                    self.players_writer.insert(*player_id, player.clone());
                }
            }
            self.players_writer.refresh();
        }
        if let Some(entity_prototypes) = &world.entity_prototypes.read() {
            for (name, entity_prototype) in entity_prototypes {
                if let Some(entity_prototype) = entity_prototype.get_one() {
                    self.entity_prototypes_writer
                        .insert(name.clone(), entity_prototype.clone());
                }
            }
            self.entity_prototypes_writer.refresh();
        } else {
            warn!("no entity_prototypes to import");
        }
        if let Some(item_prototypes) = &world.item_prototypes.read() {
            for (name, item_prototype) in item_prototypes {
                if let Some(item_prototype) = item_prototype.get_one() {
                    self.item_prototypes_writer
                        .insert(name.clone(), item_prototype.clone());
                }
            }
            self.item_prototypes_writer.refresh();
        } else {
            warn!("no item_prototypes to import");
        }
        if let Some(recipes) = &world.recipes.read() {
            for (name, recipe) in recipes {
                if let Some(recipe) = recipe.get_one() {
                    self.recipes_writer.insert(name.clone(), recipe.clone());
                }
            }
            self.recipes_writer.refresh();
        } else {
            warn!("no recipes to import");
        }
        if let Some(chunks) = &world.chunks.read() {
            for (chunk_position, chunk) in chunks {
                if let Some(chunk) = chunk.get_one() {
                    self.chunks_writer
                        .insert(chunk_position.clone(), chunk.clone());
                }
            }
            self.chunks_writer.refresh();
        } else {
            warn!("no chunks to import");
        }
        if let Some(blocked) = &world.blocked.read() {
            for (pos, minable) in blocked {
                if let Some(minable) = minable.get_one() {
                    self.blocked_writer.insert(pos.clone(), *minable);
                }
            }
            self.blocked_writer.refresh();
        } else {
            warn!("no blocked to import");
        }
        if let Some(resources) = &world.resources.read() {
            for (name, positions) in resources {
                if let Some(positions) = positions.get_one() {
                    self.resources_writer
                        .insert(name.clone(), positions.clone());
                }
            }
            self.resources_writer.refresh();
        } else {
            warn!("no resources to import");
        }
        Ok(())
    }

    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let (players_reader, players_writer) = evmap::new::<u32, FactorioPlayer>();
        let (blocked_reader, blocked_writer) = evmap::new::<Pos, bool>();
        let (chunks_reader, chunks_writer) = evmap::new::<ChunkPosition, FactorioChunk>();
        let (graphics_reader, graphics_writer) = evmap::new::<String, FactorioGraphic>();
        let (recipes_reader, recipes_writer) = evmap::new::<String, FactorioRecipe>();
        let (image_cache, image_cache_writer) = evmap::new::<String, Box<RgbaImage>>();
        let (entity_prototypes_reader, entity_prototypes_writer) =
            evmap::new::<String, FactorioEntityPrototype>();
        let (item_prototypes_reader, item_prototypes_writer) =
            evmap::new::<String, FactorioItemPrototype>();
        let (resources_reader, resources_writer) = evmap::new::<String, Vec<Position>>();
        let flow = Arc::new(Mutex::new(FlowGraph::new()));

        FactorioWorldWriter {
            players_writer,
            chunks_writer,
            graphics_writer,
            recipes_writer,
            entity_prototypes_writer,
            item_prototypes_writer,
            blocked_writer,
            resources_writer,
            flow,
            world: Arc::new(FactorioWorld {
                image_cache,
                image_cache_writer: std::sync::Mutex::new(image_cache_writer),
                blocked: blocked_reader,
                players: players_reader,
                chunks: chunks_reader,
                graphics: graphics_reader,
                recipes: recipes_reader,
                entity_prototypes: entity_prototypes_reader,
                item_prototypes: item_prototypes_reader,
                resources: resources_reader,
                actions: Mutex::new(HashMap::default()),
                path_requests: Mutex::new(HashMap::default()),
                next_action_id: Mutex::new(1),
            }),
        }
    }

    pub fn world(&self) -> Arc<FactorioWorld> {
        self.world.clone()
    }
    pub fn flow(&self) -> Arc<Mutex<FlowGraph>> {
        self.flow.clone()
    }
}
