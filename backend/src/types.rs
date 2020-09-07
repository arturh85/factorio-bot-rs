use crate::factorio::util::{add_to_rect, calculate_distance, rect_floor_ceil};
use crate::num_traits::FromPrimitive;
use evmap::ReadHandle;
use noisy_float::prelude::*;
use num_traits::ToPrimitive;
use pathfinding::utils::absdiff;
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};
use std::str::FromStr;
use typescript_definitions::TypeScriptify;

pub type FactorioInventory = HashMap<String, u32>;

#[derive(Debug, Clone, PartialEq, TypeScriptify, Serialize, Deserialize, Hash, Eq, ShallowCopy)]
#[serde(rename_all = "camelCase")]
pub struct FactorioRecipe {
    pub name: String,
    pub valid: bool,
    pub enabled: bool,
    pub category: String,
    pub ingredients: Vec<FactorioIngredient>,
    pub products: Vec<FactorioProduct>,
    pub hidden: bool,
    pub energy: Box<R64>,
    pub order: String,
    pub group: String,
    pub subgroup: String,
}

#[derive(Debug, Clone, PartialEq, TypeScriptify, Serialize, Deserialize, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FactorioBlueprintInfo {
    pub label: String,
    pub blueprint: String,
    pub width: u16,
    pub height: u16,
    pub rect: Rect,
    pub data: Value,
}

#[derive(Debug, Clone, PartialEq, TypeScriptify, Serialize, Deserialize, Hash, Eq, ShallowCopy)]
#[serde(rename_all = "camelCase")]
pub struct FactorioIngredient {
    pub name: String,
    pub ingredient_type: String,
    pub amount: u32,
}

#[derive(Debug, Clone, PartialEq, TypeScriptify, Serialize, Deserialize, Hash, Eq, ShallowCopy)]
#[serde(rename_all = "camelCase")]
pub struct FactorioProduct {
    pub name: String,
    pub product_type: String,
    pub amount: u32,
    pub probability: Box<R64>,
}

#[derive(Debug, Clone, PartialEq, TypeScriptify, Serialize, Deserialize, Hash, Eq, ShallowCopy)]
#[serde(rename_all = "camelCase")]
pub struct FactorioPlayer {
    pub player_id: u32,
    pub position: Position,
    pub main_inventory: Box<BTreeMap<String, u32>>,
    pub build_distance: u32,          // for place_entity
    pub reach_distance: u32,          // for insert_to_inventory
    pub drop_item_distance: u32,      // remove_from_inventory
    pub item_pickup_distance: u32,    // not in use, for picking up items from the ground
    pub loot_pickup_distance: u32, // not in use, for picking up items from the ground automatically
    pub resource_reach_distance: u32, // for mine
}

impl Default for FactorioPlayer {
    fn default() -> Self {
        FactorioPlayer {
            player_id: 0,
            position: Position::default(),
            main_inventory: Box::new(BTreeMap::new()),
            build_distance: 10,
            reach_distance: 10,
            drop_item_distance: 10,
            item_pickup_distance: 1,
            loot_pickup_distance: 2,
            resource_reach_distance: 3,
        }
    }
}

#[derive(Debug, Clone, PartialEq, TypeScriptify, Serialize, Deserialize, Hash, Eq, ShallowCopy)]
#[serde(rename_all = "camelCase")]
pub struct RequestEntity {
    pub name: String,
    pub position: Position,
}

#[derive(Debug, Clone, PartialEq, TypeScriptify, Serialize, Deserialize, Hash, Eq, ShallowCopy)]
#[serde(rename_all = "camelCase")]
pub struct InventoryResponse {
    pub name: String,
    pub position: Position,
    pub output_inventory: Box<Option<BTreeMap<String, u32>>>,
    pub fuel_inventory: Box<Option<BTreeMap<String, u32>>>,
}

#[derive(Debug, Clone, PartialEq, TypeScriptify, Serialize, Deserialize, Hash, Eq, ShallowCopy)]
#[serde(rename_all = "camelCase")]
pub struct ChunkPosition {
    pub x: i32,
    pub y: i32,
}

impl From<&Pos> for ChunkPosition {
    fn from(pos: &Pos) -> ChunkPosition {
        ChunkPosition {
            x: pos.0 / 32,
            y: pos.1 / 32,
        }
    }
}

#[derive(Debug, Clone, PartialEq, TypeScriptify, Serialize, Deserialize, Hash, Eq, ShallowCopy)]
#[serde(rename_all = "camelCase")]
pub struct Position {
    pub x: Box<R64>,
    pub y: Box<R64>,
}

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "[{}, {}]",
            (self.x() * 100.).round() / 100.,
            (self.y() * 100.).round() / 100.
        ))?;
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Pos(pub i32, pub i32);

impl Pos {
    pub fn distance(&self, other: &Pos) -> u32 {
        (absdiff(self.0, other.0) + absdiff(self.1, other.1)) as u32
    }
}

impl From<&Position> for Pos {
    fn from(position: &Position) -> Pos {
        Pos(position.x().floor() as i32, position.y().floor() as i32)
    }
}

impl From<&Pos> for Position {
    fn from(pos: &Pos) -> Position {
        Position::new(pos.0 as f64, pos.1 as f64)
    }
}

#[derive(Primitive, Clone, Copy, Debug, PartialEq)]
pub enum Direction {
    North = 0,
    NorthEast = 1,
    East = 2,
    SouthEast = 3,
    South = 4,
    SouthWest = 5,
    West = 6,
    NorthWest = 7,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AreaFilter {
    Rect(Rect),
    PositionRadius((Position, Option<f64>)),
}

impl Direction {
    pub fn all() -> Vec<Direction> {
        (0..8).map(|n| Direction::from_u8(n).unwrap()).collect()
    }
    pub fn orthogonal() -> Vec<Direction> {
        (0..8)
            .filter(|n| n % 2 == 0)
            .map(|n| Direction::from_u8(n).unwrap())
            .collect()
    }
    pub fn opposite(&self) -> Direction {
        Direction::from_u8((Direction::to_u8(self).unwrap() + 4) % 8).unwrap()
    }
    pub fn clockwise(&self) -> Direction {
        Direction::from_u8((Direction::to_u8(self).unwrap() + 2) % 8).unwrap()
    }
}

impl Default for Direction {
    fn default() -> Self {
        Direction::North
    }
}

impl Position {
    pub fn new(x: f64, y: f64) -> Position {
        Position {
            x: Box::new(r64(x)),
            y: Box::new(r64(y)),
        }
    }

    pub fn x(&self) -> f64 {
        (*self.x).to_f64().expect("failed to cast r64 to f64")
    }
    pub fn y(&self) -> f64 {
        (*self.y).to_f64().expect("failed to cast r64 to f64")
    }
    pub fn add(&self, position: &Position) -> Position {
        Position::new(self.x() + position.x(), self.y() + position.y())
    }

    pub fn turn(&self, direction: Direction) -> Position {
        match direction {
            Direction::North => self.clone(),
            Direction::East => self.rotate_clockwise(),
            Direction::South => self.rotate_clockwise().rotate_clockwise(),
            Direction::West => self
                .rotate_clockwise()
                .rotate_clockwise()
                .rotate_clockwise(),
            _ => panic!("diagonal turning not supported"),
        }
    }

    /*
    https://limnu.com/sketch-easy-90-degree-rotate-vectors/#:~:text=Normally%20rotating%20vectors%20involves%20matrix,swap%20X%20and%20Y%20values.
    Normally rotating vectors involves matrix math, but there’s a really simple trick for rotating a 2D vector by 90° clockwise:
    just multiply the X part of the vector by -1, and then swap X and Y values.
     */
    pub fn rotate_clockwise(&self) -> Position {
        Position::new(self.y(), self.x() * -1.0)
    }
}

impl Default for Position {
    fn default() -> Self {
        Position::new(0., 0.)
    }
}

impl FromStr for Position {
    type Err = anyhow::Error;
    fn from_str(str: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = str.split(',').collect();
        if parts.len() != 2 {
            return Err(anyhow!(
                "invalid position: expected A,B like 1.2,3.4 got {}",
                str
            ));
        }
        Ok(Position::new(parts[0].parse()?, parts[1].parse()?))
    }
}

#[derive(
    Debug, Clone, Default, PartialEq, TypeScriptify, Serialize, Deserialize, Hash, Eq, ShallowCopy,
)]
#[serde(rename_all = "camelCase")]
pub struct Rect {
    pub left_top: Position,
    pub right_bottom: Position,
}

impl Rect {
    pub fn new(left_top: &Position, right_bottom: &Position) -> Rect {
        Rect {
            left_top: left_top.clone(),
            right_bottom: right_bottom.clone(),
        }
    }
    pub fn contains(&self, position: &Position) -> bool {
        position.x() > self.left_top.x()
            && position.x() < self.right_bottom.x()
            && position.y() > self.left_top.y()
            && position.y() < self.right_bottom.y()
    }

    pub fn from_wh(width: f64, height: f64) -> Rect {
        Rect {
            left_top: Position::new(-width / 2., -height / 2.),
            right_bottom: Position::new(width / 2., height / 2.),
        }
    }
    pub fn width(&self) -> f64 {
        self.right_bottom.x() - self.left_top.x()
    }
    pub fn height(&self) -> f64 {
        self.right_bottom.y() - self.left_top.y()
    }
    pub fn center(&self) -> Position {
        Position::new(
            (self.left_top.x() + self.right_bottom.x()) / 2.,
            (self.left_top.y() + self.right_bottom.y()) / 2.,
        )
    }
}

impl FromStr for Rect {
    type Err = anyhow::Error;
    fn from_str(str: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = str.split(';').collect();
        if parts.len() != 2 {
            return Err(anyhow!(
                "invalid rect: expected A,B;C,D like 1.2,3.4;5.6,7.8 got {}",
                str
            ));
        }
        Ok(Rect {
            left_top: parts[0].parse()?,
            right_bottom: parts[1].parse()?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, TypeScriptify, Serialize, Deserialize, Hash, Eq, ShallowCopy)]
#[serde(rename_all = "camelCase")]
pub struct FactorioTile {
    pub name: String,
    pub player_collidable: bool,
    pub position: Position,
}

#[derive(
    Debug, Clone, Default, PartialEq, TypeScriptify, Serialize, Deserialize, Hash, Eq, ShallowCopy,
)]
#[serde(rename_all = "camelCase")]
pub struct FactorioChunk {
    pub entities: Vec<FactorioEntity>,
    pub tiles: Vec<FactorioTile>,
}

#[derive(Debug, Clone, PartialEq, TypeScriptify, Serialize, Deserialize, Hash, Eq, ShallowCopy)]
#[serde(rename_all = "camelCase")]
pub struct ChunkObject {
    pub name: String,
    pub position: Position,
    pub direction: String,
    pub bounding_box: Rect,
    pub output_inventory: Box<Option<BTreeMap<String, u32>>>,
    pub fuel_inventory: Box<Option<BTreeMap<String, u32>>>,
}

#[derive(Debug, Clone, PartialEq, TypeScriptify, Serialize, Deserialize, Hash, Eq, ShallowCopy)]
#[serde(rename_all = "camelCase")]
pub struct ChunkResource {
    pub name: String,
    pub position: Position,
}

#[derive(Debug, Clone, PartialEq, TypeScriptify, Serialize, Deserialize, Hash, Eq, ShallowCopy)]
#[serde(rename_all = "camelCase")]
pub struct FactorioTechnology {
    pub name: String,
    pub enabled: bool,
    pub upgrade: bool,
    pub researched: bool,
    pub prerequisites: Option<Vec<String>>,
    pub research_unit_ingredients: Vec<FactorioIngredient>,
    pub research_unit_count: u32,
    pub research_unit_energy: Box<R64>,
    pub order: String,
    pub level: u32,
    pub valid: bool,
}

#[derive(Debug, Clone, PartialEq, TypeScriptify, Serialize, Deserialize, Hash, Eq, ShallowCopy)]
#[serde(rename_all = "camelCase")]
pub struct FactorioForce {
    pub name: String,
    pub force_id: u32,
    // The current technology in research, or None if no research is currently ongoing.
    pub current_research: Option<String>,
    // Progress of current research, as a number in range [0, 1].
    pub research_progress: Option<Box<R64>>,
    pub technologies: Box<BTreeMap<String, FactorioTechnology>>,
}

#[derive(Debug, Clone, PartialEq, TypeScriptify, Serialize, Deserialize, Hash, Eq, ShallowCopy)]
#[serde(rename_all = "camelCase")]
pub struct FactorioGraphic {
    pub entity_name: String,
    pub image_path: String,
    pub width: u32,
    pub height: u32, // FIXME: add whatever this is, width&height are the first
                     // 1:1:0:0:0:0:1

                     //picspec.filename..":"..picspec.width..":"..picspec.height..":"..shiftx..":"..shifty..":"..xx..":"..yy..":"..scale
}

#[derive(Debug, Clone, PartialEq, TypeScriptify, Serialize, Deserialize, Hash, Eq, ShallowCopy)]
#[serde(rename_all = "camelCase")]
pub struct FactorioEntityPrototype {
    pub name: String,
    pub entity_type: String,
    pub collision_mask: Option<Vec<String>>,
    pub collision_box: Rect,
    pub mine_result: Box<Option<BTreeMap<String, u32>>>,
    pub mining_time: Box<Option<R64>>,
}

#[derive(
    Debug, Clone, Default, PartialEq, TypeScriptify, Serialize, Deserialize, Hash, Eq, ShallowCopy,
)]
#[serde(rename_all = "camelCase")]
pub struct FactorioEntity {
    pub name: String,
    pub entity_type: String,
    pub position: Position,
    pub bounding_box: Rect,
    pub direction: u8,
    pub drop_position: Option<Position>,
    pub pickup_position: Option<Position>, // only type = inserter
    pub output_inventory: Box<Option<BTreeMap<String, u32>>>,
    pub fuel_inventory: Box<Option<BTreeMap<String, u32>>>,
    pub amount: Option<u32>,        // only type = resource
    pub recipe: Option<String>,     // only CraftingMachines
    pub ghost_name: Option<String>, // only type = entity-ghost
    pub ghost_type: Option<String>, // only type = entity-ghost
}

impl FactorioEntity {
    pub fn new_burner_mining_drill(position: &Position, direction: Direction) -> FactorioEntity {
        FactorioEntity {
            name: EntityName::BurnerMiningDrill.to_string(),
            entity_type: EntityType::MiningDrill.to_string(),
            position: position.clone(),
            bounding_box: add_to_rect(&Rect::from_wh(1.8, 1.8), &position),
            direction: direction.to_u8().unwrap(),
            drop_position: Some(position.add(&Position::new(-0.5, -1.296875).turn(direction))),
            ..Default::default()
        }
    }
    pub fn new_stone_furnace(position: &Position, direction: Direction) -> FactorioEntity {
        FactorioEntity {
            name: EntityName::StoneFurnace.to_string(),
            entity_type: EntityType::Furnace.to_string(),
            position: position.clone(),
            bounding_box: add_to_rect(&Rect::from_wh(1.8, 1.8), &position),
            direction: direction.to_u8().unwrap(),
            ..Default::default()
        }
    }

    pub fn is_minable(&self) -> bool {
        self.entity_type == EntityType::Tree.to_string()
            || self.entity_type == EntityType::SimpleEntity.to_string()
    }
    pub fn is_fluid_input(&self) -> bool {
        self.entity_type == EntityType::Pipe.to_string()
            || self.entity_type == EntityType::PipeToGround.to_string()
            || self.entity_type == EntityType::Boiler.to_string()
    }
}

#[derive(Debug, Clone, PartialEq, TypeScriptify, Serialize, Deserialize, Hash, Eq, ShallowCopy)]
#[serde(rename_all = "camelCase")]
pub struct FactorioItemPrototype {
    pub name: String,
    pub item_type: String,
    pub stack_size: u32,
    pub fuel_value: u64,
    pub place_result: String,
    pub group: String,
    pub subgroup: String,
}

#[derive(EnumString, Display, Debug)]
#[strum(serialize_all = "kebab-case")]
pub enum EntityName {
    Wood,
    Stone,
    Coal,
    IronOre,
    CopperOre,
    UraniumOre,
    StoneFurnace,
    BurnerMiningDrill,
}

#[derive(EnumString, Display, Debug, PartialEq, Clone)]
#[strum(serialize_all = "kebab-case")]
pub enum EntityType {
    None,
    Assembler,
    Boiler,
    Container,
    Resource,
    SimpleEntity,
    Tree,
    Inserter,
    MiningDrill,
    Furnace,
    TransportBelt,
    Splitter,
    UndergroundBelt,
    Pipe,
    PipeToGround,
    OffshorePump,
}

impl Default for EntityType {
    fn default() -> Self {
        EntityType::None
    }
}

#[derive(Debug)]
pub struct ResourcePatch {
    pub name: String,
    pub id: u32,
    pub rect: Rect,
    pub elements: Vec<Position>,
}

impl ResourcePatch {
    pub fn contains(&self, pos: Pos) -> bool {
        let field: Vec<Pos> = self.elements.iter().map(|e| e.into()).collect();
        field.contains(&pos)
    }
    pub fn find_free_rect(
        &self,
        width: u32,
        height: u32,
        near: &Position,
        blocked: &ReadHandle<Pos, bool>,
    ) -> Option<Rect> {
        let mut elements = self.elements.clone();
        elements.sort_by(|a, b| {
            let da = r64(calculate_distance(&a, &near));
            let db = r64(calculate_distance(&b, &near));
            da.cmp(&db)
        });

        let mut element_map: HashMap<Pos, bool> = HashMap::new();
        for element in &elements {
            element_map.insert(element.into(), true);
        }
        for element in &elements {
            let mut invalid = false;
            for y in 0i32..height as i32 {
                for x in 0i32..width as i32 {
                    let pos = Pos(element.x() as i32 + x, element.y() as i32 + y);
                    let blocked_pos = blocked.get_one(&pos);
                    if blocked_pos.is_some() && !*blocked_pos.unwrap() {
                        invalid = true;
                        break;
                    }
                    if element_map.get(&pos).is_none() {
                        invalid = true;
                        break;
                    }
                }
                if invalid {
                    break;
                }
            }
            if !invalid {
                return Some(rect_floor_ceil(&Rect {
                    left_top: element.clone(),
                    right_bottom: Position::new(
                        element.x() + width as f64,
                        element.y() + height as f64,
                    ),
                }));
            }
        }
        None
    }
}

#[derive(Debug, Clone, PartialEq, TypeScriptify, Serialize, Deserialize, Hash, Eq, ShallowCopy)]
#[serde(rename_all = "camelCase")]
pub struct FactorioResult {
    pub success: bool,
    pub output: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, TypeScriptify, Serialize, Deserialize, Hash, Eq, ShallowCopy)]
#[serde(rename_all = "camelCase")]
pub struct PlaceEntityResult {
    pub player: FactorioPlayer,
    pub entity: FactorioEntity,
}
#[derive(Debug, Clone, PartialEq, TypeScriptify, Serialize, Deserialize, Hash, Eq, ShallowCopy)]
#[serde(rename_all = "camelCase")]
pub struct PlaceEntitiesResult {
    pub player: FactorioPlayer,
    pub entities: Vec<FactorioEntity>,
}

#[derive(Debug, Clone, PartialEq, TypeScriptify, Serialize, Deserialize, Hash, Eq, ShallowCopy)]
#[serde(rename_all = "camelCase")]
pub struct PlayerChangedDistanceEvent {
    pub player_id: u32,
    pub build_distance: u32,
    pub reach_distance: u32,
    pub drop_item_distance: u32,
    pub item_pickup_distance: u32,
    pub loot_pickup_distance: u32,
    pub resource_reach_distance: u32,
}

#[derive(Debug, Clone, PartialEq, TypeScriptify, Serialize, Deserialize, Hash, Eq, ShallowCopy)]
#[serde(rename_all = "camelCase")]
pub struct PlayerChangedPositionEvent {
    pub player_id: u32,
    pub position: Position,
}

#[derive(Debug, Clone, PartialEq, TypeScriptify, Serialize, Deserialize, Hash, Eq, ShallowCopy)]
#[serde(rename_all = "camelCase")]
pub struct PlayerChangedMainInventoryEvent {
    pub player_id: u32,
    pub main_inventory: Box<BTreeMap<String, u32>>,
}

#[derive(Debug, Clone, PartialEq, TypeScriptify, Serialize, Deserialize, Hash, Eq, ShallowCopy)]
#[serde(rename_all = "camelCase")]
pub struct PlayerLeftEvent {
    pub player_id: u32,
}
