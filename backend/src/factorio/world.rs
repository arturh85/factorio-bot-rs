use crate::factorio::entity_graph::{entity_node_at, EntityGraph, EntityNode};
use crate::factorio::util::{
    bounding_box, move_position, position_equal, rect_fields, rect_floor, rect_floor_ceil,
};
use crate::types::{
    ChunkPosition, Direction, EntityName, EntityType, FactorioChunk, FactorioEntity,
    FactorioEntityPrototype, FactorioForce, FactorioGraphic, FactorioItemPrototype, FactorioPlayer,
    FactorioRecipe, FactorioTile, PlayerChangedDistanceEvent, PlayerChangedMainInventoryEvent,
    PlayerChangedPositionEvent, Pos, Position, ResourcePatch,
};
use async_std::sync::Mutex;
use evmap::{ReadHandle, WriteHandle};
use image::RgbaImage;
use petgraph::graph::{EdgeIndex, NodeIndex};
use petgraph::visit::EdgeRef;
use std::collections::{BTreeMap, HashMap};
use std::str::FromStr;
use std::sync::Arc;

#[derive(Debug)]
pub struct FactorioWorld {
    pub players: ReadHandle<u32, FactorioPlayer>,
    pub forces: ReadHandle<String, FactorioForce>,
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

impl FactorioWorld {
    fn walk(&self, m: &mut HashMap<Position, Option<u32>>, pos: &Position, id: u32) {
        m.insert(pos.clone(), Some(id));
        for direction in Direction::all() {
            let other = move_position(pos, direction, 1.0);
            if let Some(p) = m.get(&other) {
                if p.is_none() {
                    self.walk(m, &other, id);
                }
            }
        }
    }
    pub fn resource_contains(&self, resource_name: &str, pos: Pos) -> bool {
        let elements = self.resources.get_one(resource_name);
        if let Some(elements) = elements {
            let field: Vec<Pos> = elements.iter().map(|e| e.into()).collect();
            field.contains(&pos)
        } else {
            false
        }
    }

    pub fn resource_patches(&self, resource_name: &str) -> Vec<ResourcePatch> {
        let mut patches: Vec<ResourcePatch> = vec![];
        let mut positions_by_id: HashMap<Position, Option<u32>> = HashMap::new();
        for point in self
            .resources
            .get_one(resource_name)
            .expect("resource patch not found")
            .iter()
        {
            positions_by_id.insert(point.clone(), None);
        }
        let mut next_id: u32 = 0;

        while let Some((next_pos, _)) = positions_by_id.iter().find(|(_, value)| value.is_none()) {
            next_id += 1;
            let next_pos = next_pos.clone();
            self.walk(&mut positions_by_id, &next_pos, next_id);
        }
        for id in 1..=next_id {
            let mut elements: Vec<Position> = vec![];
            for (k, v) in &positions_by_id {
                if v.unwrap() == id {
                    elements.push(k.clone());
                }
            }
            patches.push(ResourcePatch {
                name: resource_name.into(),
                rect: bounding_box(&elements).unwrap(),
                elements,
                id,
            });
        }
        patches.sort_by(|a, b| b.elements.len().cmp(&a.elements.len()));
        patches
    }
}

unsafe impl Send for FactorioWorld {}
unsafe impl Sync for FactorioWorld {}

pub struct FactorioWorldWriter {
    pub world: Arc<FactorioWorld>,
    forces_writer: WriteHandle<String, FactorioForce>,
    chunks_writer: WriteHandle<ChunkPosition, FactorioChunk>,
    graphics_writer: WriteHandle<String, FactorioGraphic>,
    recipes_writer: WriteHandle<String, FactorioRecipe>,
    entity_prototypes_writer: WriteHandle<String, FactorioEntityPrototype>,
    item_prototypes_writer: WriteHandle<String, FactorioItemPrototype>,
    players_writer: WriteHandle<u32, FactorioPlayer>,
    blocked_writer: WriteHandle<Pos, bool>,
    resources_writer: WriteHandle<String, Vec<Position>>,
    entity_graph: Arc<std::sync::Mutex<EntityGraph>>,
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

    pub fn connect_entity_graph(&mut self) -> anyhow::Result<()> {
        let mut entity_graph = self.entity_graph.lock().unwrap();
        let mut edges_to_add: Vec<(NodeIndex, NodeIndex, f64)> = vec![];
        for node_index in entity_graph.node_indices() {
            let node_index = node_index;
            let node = entity_graph.node_weight(node_index).unwrap();
            if let Some(drop_position) = node.entity.drop_position.as_ref() {
                match entity_node_at(&entity_graph, drop_position) {
                    Some(drop_index) => {
                        if !entity_graph.contains_edge(node_index, drop_index) {
                            edges_to_add.push((node_index, drop_index, 1.));
                        }
                    }
                    None => error!(
                        "connect entity graph could not find entity at Drop position {} for {:?}",
                        drop_position, &node.entity
                    ),
                }
            }
            if let Some(pickup_position) = node.entity.pickup_position.as_ref() {
                match entity_node_at(&entity_graph, pickup_position) {
                    Some(pickup_index) => {
                        if !entity_graph.contains_edge(pickup_index, node_index) {
                            edges_to_add.push((pickup_index, node_index, 1.));
                        }
                    }
                    None => error!(
                        "connect entity graph could not find entity at Pickup position {} for {:?}",
                        pickup_position, &node.entity
                    ),
                }
            }
            if EntityType::Splitter == node.entity_type {
                let in1 = node
                    .entity
                    .position
                    .add(&Position::new(-0.5, 1.).turn(node.direction));
                let in2 = node
                    .entity
                    .position
                    .add(&Position::new(0.5, 1.).turn(node.direction));
                for pos in vec![in1, in2] {
                    if let Some(prev_index) = entity_node_at(&entity_graph, &pos) {
                        let prev = entity_graph.node_weight(prev_index).unwrap();
                        if (prev.entity_type == EntityType::TransportBelt
                            || prev.entity_type == EntityType::UndergroundBelt
                            || prev.entity_type == EntityType::Splitter)
                            && !entity_graph.contains_edge(node_index, prev_index)
                        {
                            edges_to_add.push((node_index, prev_index, 1.));
                        }
                    }
                }
            } else if EntityType::TransportBelt == node.entity_type {
                if let Some(next_index) = entity_node_at(
                    &entity_graph,
                    &move_position(&node.entity.position, node.direction, 1.0),
                ) {
                    let next = entity_graph.node_weight(next_index).unwrap();
                    if (next.entity_type == EntityType::TransportBelt
                        || next.entity_type == EntityType::UndergroundBelt
                        || next.entity_type == EntityType::Splitter)
                        && !entity_graph.contains_edge(node_index, next_index)
                    {
                        edges_to_add.push((node_index, next_index, 1.));
                    }
                }
            } else if EntityType::OffshorePump == node.entity_type {
                if let Some(next_index) = entity_node_at(
                    &entity_graph,
                    &move_position(&node.entity.position, node.direction, -1.),
                ) {
                    let next = entity_graph.node_weight(next_index).unwrap();
                    if next.entity.is_fluid_input()
                        && !entity_graph.contains_edge(node_index, next_index)
                    {
                        edges_to_add.push((node_index, next_index, 1.));
                    }
                }
            } else if EntityType::Pipe == node.entity_type {
                for direction in Direction::orthogonal() {
                    if let Some(next_index) = entity_node_at(
                        &entity_graph,
                        &move_position(&node.entity.position, direction, 1.),
                    ) {
                        let next = entity_graph.node_weight(next_index).unwrap();
                        if next.entity.is_fluid_input() {
                            if !entity_graph.contains_edge(node_index, next_index) {
                                edges_to_add.push((node_index, next_index, 1.));
                            }
                            if !entity_graph.contains_edge(next_index, node_index) {
                                edges_to_add.push((next_index, node_index, 1.));
                            }
                        }
                    }
                }
            } else if EntityType::UndergroundBelt == node.entity_type {
                let mut found = false;
                for length in 1..5 {
                    // todo: lookup in EntityPrototypes for real belt length
                    if let Some(next_index) = entity_node_at(
                        &entity_graph,
                        &move_position(
                            &node.entity.position,
                            node.direction.opposite(),
                            length as f64,
                        ),
                    ) {
                        let next = entity_graph.node_weight(next_index).unwrap();
                        if next.entity_type == EntityType::UndergroundBelt {
                            if !entity_graph.contains_edge(next_index, node_index) {
                                edges_to_add.push((next_index, node_index, length as f64));
                            }
                            found = true;
                            break;
                        }
                    }
                }
                if found {
                    if let Some(next_index) = entity_node_at(
                        &entity_graph,
                        &move_position(&node.entity.position, node.direction, 1.),
                    ) {
                        let next = entity_graph.node_weight(next_index).unwrap();
                        if (next.entity_type == EntityType::TransportBelt
                            || next.entity_type == EntityType::Splitter)
                            && !entity_graph.contains_edge(node_index, next_index)
                        {
                            edges_to_add.push((node_index, next_index, 1.));
                        }
                    }
                }
            } else if EntityType::PipeToGround == node.entity_type {
                let mut found = false;
                for length in 1..12 {
                    // todo: lookup in EntityPrototypes for real pipe length
                    if let Some(next_index) = entity_node_at(
                        &entity_graph,
                        &move_position(&node.entity.position, node.direction, -length as f64),
                    ) {
                        let next = entity_graph.node_weight(next_index).unwrap();
                        if next.entity_type == EntityType::PipeToGround {
                            if !entity_graph.contains_edge(next_index, node_index) {
                                edges_to_add.push((next_index, node_index, length as f64));
                            }
                            if !entity_graph.contains_edge(node_index, next_index) {
                                edges_to_add.push((node_index, next_index, length as f64));
                            }
                            found = true;
                            break;
                        }
                    }
                }
                if found {
                    if let Some(next_index) = entity_node_at(
                        &entity_graph,
                        &move_position(&node.entity.position, node.direction, 1.),
                    ) {
                        let next = entity_graph.node_weight(next_index).unwrap();
                        if next.entity.is_fluid_input()
                            && !entity_graph.contains_edge(node_index, next_index)
                        {
                            edges_to_add.push((node_index, next_index, 1.));
                        }
                    }
                }
            }
        }
        for (a, b, w) in edges_to_add {
            if !entity_graph.contains_edge(a, b) {
                entity_graph.add_edge(a, b, w);
            }
        }
        Ok(())
    }

    fn add_to_entity_graph(&mut self, entity: &FactorioEntity) -> anyhow::Result<()> {
        if let Ok(entity_type) = EntityType::from_str(&entity.entity_type) {
            match entity_type {
                EntityType::Furnace
                | EntityType::Inserter
                | EntityType::Boiler
                | EntityType::OffshorePump
                | EntityType::MiningDrill
                | EntityType::Container
                | EntityType::Splitter
                | EntityType::TransportBelt
                | EntityType::UndergroundBelt
                | EntityType::Pipe
                | EntityType::PipeToGround
                | EntityType::Assembler => {
                    let mut entity_graph = self.entity_graph.lock().unwrap();
                    if entity_node_at(&entity_graph, &entity.position).is_none() {
                        let miner_ore = if entity_type == EntityType::MiningDrill {
                            let rect = rect_floor(&entity.bounding_box);
                            let mut miner_ore: Option<String> = None;
                            for resource in &[
                                EntityName::IronOre,
                                EntityName::CopperOre,
                                EntityName::Coal,
                                EntityName::Stone,
                                EntityName::UraniumOre,
                            ] {
                                let resource = resource.to_string();
                                let resource_found = rect_fields(&rect)
                                    .iter()
                                    .any(|p| self.world.resource_contains(&resource, p.into()));
                                if resource_found {
                                    miner_ore = Some(resource);
                                    break;
                                }
                            }
                            if miner_ore.is_none() {
                                warn!("no ore found under miner {:?}", &entity);
                            }
                            miner_ore
                        } else {
                            None
                        };
                        entity_graph.add_node(EntityNode::new(entity.clone(), miner_ore));
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn remove_from_entity_graph(&mut self, entity: &FactorioEntity) -> anyhow::Result<()> {
        let mut entity_graph = self.entity_graph.lock().unwrap();

        let mut nodes_to_remove: Vec<NodeIndex> = vec![];
        let mut edges_to_remove: Vec<EdgeIndex> = vec![];

        if let Some(node_index) = entity_node_at(&entity_graph, &entity.position) {
            for edge in entity_graph.edges_directed(node_index, petgraph::Direction::Incoming) {
                edges_to_remove.push(edge.id());
            }
            for edge in entity_graph.edges_directed(node_index, petgraph::Direction::Outgoing) {
                edges_to_remove.push(edge.id());
            }
            nodes_to_remove.push(node_index);
        }
        for edge in edges_to_remove {
            entity_graph.remove_edge(edge);
        }
        for node in nodes_to_remove {
            entity_graph.remove_node(node);
        }

        Ok(())
    }

    pub fn update_force(&mut self, force: FactorioForce) -> anyhow::Result<()> {
        let name = force.name.clone();
        self.forces_writer.insert(name, force);
        self.forces_writer.refresh();
        Ok(())
    }

    pub fn on_some_entity_created(&mut self, entity: FactorioEntity) -> anyhow::Result<()> {
        let rect = rect_floor(&entity.bounding_box);
        for position in rect_fields(&rect) {
            self.blocked_writer
                .insert((&position).into(), entity.is_minable());
        }
        let pos: Pos = (&entity.position).into();
        let chunk_position: ChunkPosition = (&pos).into();
        self.add_to_entity_graph(&entity)?;
        match self.world.chunks.get_one(&chunk_position) {
            Some(chunk) => {
                let mut chunk = chunk.clone();
                chunk.entities.push(entity);
                self.chunks_writer.update(chunk_position, chunk);
            }
            None => {
                self.chunks_writer.insert(
                    chunk_position,
                    FactorioChunk {
                        entities: vec![entity],
                        ..Default::default()
                    },
                );
            }
        }
        self.blocked_writer.refresh();
        self.chunks_writer.refresh();
        Ok(())
    }

    pub fn on_some_entity_deleted(&mut self, entity: FactorioEntity) -> anyhow::Result<()> {
        let rect = rect_floor(&entity.bounding_box);
        for position in rect_fields(&rect) {
            self.blocked_writer.clear((&position).into());
        }
        let pos: Pos = (&entity.position).into();
        let chunk_position: ChunkPosition = (&pos).into();
        self.remove_from_entity_graph(&entity)?;
        if let Some(chunk) = self.world.chunks.get_one(&chunk_position) {
            let mut chunk = chunk.clone();
            if let Some(index) = chunk
                .entities
                .iter()
                .position(|en| position_equal(&en.position, &entity.position))
            {
                chunk.entities.remove(index);
            }
            self.chunks_writer.update(chunk_position, chunk);
        }
        self.blocked_writer.refresh();
        self.chunks_writer.refresh();
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
                    tiles,
                    ..Default::default()
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
                _ => {
                    let rect = rect_floor_ceil(&entity.bounding_box);
                    for position in rect_fields(&rect) {
                        self.blocked_writer
                            .insert((&position).into(), entity.is_minable());
                    }
                }
            }
        }

        if self.chunks_writer.contains_key(&chunk_position) {
            let existing_chunk = self.chunks_writer.get_one(&chunk_position).unwrap(); // unwrap OK because of contains_key
            let chunk = FactorioChunk {
                entities: entities.clone(),
                tiles: existing_chunk.tiles.clone(),
            };
            drop(existing_chunk);
            self.chunks_writer.empty(chunk_position.clone());
            self.chunks_writer.insert(chunk_position.clone(), chunk);
        } else {
            self.chunks_writer.insert(
                chunk_position,
                FactorioChunk {
                    entities: entities.clone(),
                    ..Default::default()
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
        for entity in &entities {
            self.add_to_entity_graph(entity)?;
        }
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
        if let Some(forces) = &world.forces.read() {
            for (name, force) in forces {
                if let Some(force) = force.get_one() {
                    self.forces_writer.insert(name.clone(), force.clone());
                }
            }
            self.forces_writer.refresh();
        } else {
            warn!("no forces to import");
        }
        if let Some(chunks) = &world.chunks.read() {
            for (chunk_position, chunk) in chunks {
                if let Some(chunk) = chunk.get_one() {
                    let FactorioChunk { entities, tiles } = chunk;
                    self.update_chunk_entities(chunk_position.clone(), entities.clone())?;
                    self.update_chunk_tiles(chunk_position.clone(), tiles.clone())?;
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
        self.connect_entity_graph()?;
        Ok(())
    }

    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let (forces_reader, forces_writer) = evmap::new::<String, FactorioForce>();
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
        let entity_graph = Arc::new(std::sync::Mutex::new(EntityGraph::new()));

        FactorioWorldWriter {
            forces_writer,
            players_writer,
            chunks_writer,
            graphics_writer,
            recipes_writer,
            entity_prototypes_writer,
            item_prototypes_writer,
            blocked_writer,
            resources_writer,
            entity_graph,
            world: Arc::new(FactorioWorld {
                image_cache,
                image_cache_writer: std::sync::Mutex::new(image_cache_writer),
                blocked: blocked_reader,
                players: players_reader,
                chunks: chunks_reader,
                graphics: graphics_reader,
                recipes: recipes_reader,
                forces: forces_reader,
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
    pub fn entity_graph(&self) -> Arc<std::sync::Mutex<EntityGraph>> {
        self.entity_graph.clone()
    }
}
