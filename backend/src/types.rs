use noisy_float::prelude::*;
use num_traits::ToPrimitive;
use std::collections::BTreeMap;
use std::str::FromStr;
use typescript_definitions::TypeScriptify;

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
    pub build_distance: Option<u32>,
    pub reach_distance: Option<u32>,
    pub drop_item_distance: Option<u32>,
    pub item_pickup_distance: Option<u32>,
    pub loot_pickup_distance: Option<u32>,
    pub resource_reach_distance: Option<u32>,
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

#[derive(Debug, Clone, PartialEq, TypeScriptify, Serialize, Deserialize, Hash, Eq, ShallowCopy)]
#[serde(rename_all = "camelCase")]
pub struct Position {
    pub x: Box<R64>,
    pub y: Box<R64>,
}

#[derive(Primitive)]
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

#[derive(Debug, Clone, PartialEq, TypeScriptify, Serialize, Deserialize, Hash, Eq, ShallowCopy)]
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
    pub fn width(&self) -> f64 {
        self.right_bottom.x() - self.left_top.x()
    }
    pub fn height(&self) -> f64 {
        self.right_bottom.y() - self.left_top.y()
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

#[derive(Debug, Clone, PartialEq, TypeScriptify, Serialize, Deserialize, Hash, Eq, ShallowCopy)]
#[serde(rename_all = "camelCase")]
pub struct FactorioChunk {
    pub objects: Vec<ChunkObject>,
    pub resources: Vec<ChunkResource>,
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
}

#[derive(Debug, Clone, PartialEq, TypeScriptify, Serialize, Deserialize, Hash, Eq, ShallowCopy)]
#[serde(rename_all = "camelCase")]
pub struct FactorioEntity {
    pub name: String,
    pub entity_type: String,
    pub position: Position,
    pub amount: Option<u32>,        // only type = resource
    pub recipe: Option<String>,     // only CraftingMachines
    pub ghost_name: Option<String>, // only type = entity-ghost
    pub ghost_type: Option<String>, // only type = entity-ghost
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

#[derive(Debug, Clone, PartialEq, TypeScriptify, Serialize, Deserialize, Hash, Eq, ShallowCopy)]
#[serde(rename_all = "camelCase")]
pub struct FactorioResult {
    pub success: bool,
    pub output: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Hash, Eq, ShallowCopy)]
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Hash, Eq, ShallowCopy)]
#[serde(rename_all = "camelCase")]
pub struct PlayerChangedPositionEvent {
    pub player_id: u32,
    pub position: Position,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Hash, Eq, ShallowCopy)]
#[serde(rename_all = "camelCase")]
pub struct PlayerMainInventoryChangedEvent {
    pub player_id: u32,
    pub main_inventory: Box<BTreeMap<String, u32>>,
}
