use crate::types::{
    FactorioEntity, FactorioEntityPrototype, FactorioForce, FactorioItemPrototype, FactorioPlayer,
    FactorioRecipe, FactorioTile, InventoryResponse, PlaceEntitiesResult, PlaceEntityResult,
    Position, RequestEntity,
};
use std::collections::HashMap;
use std::sync::Arc;

use crate::factorio::output_parser::FactorioWorld;
use crate::factorio::rcon::FactorioRcon;
use actix_web::http::StatusCode;
use actix_web::web::{Json, Path as PathInfo};
use actix_web::{web, HttpResponse};
use factorio_blueprint::BlueprintCodec;
use serde::export::Formatter;
use serde_json::Value;
use std::fmt::Display;
use std::time::Duration;

#[derive(Debug)]
pub struct MyError {
    err: anyhow::Error,
}
impl Display for MyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.write_str("ERROR")?;
        Ok(())
    }
}

impl actix_web::error::ResponseError for MyError {
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }

    fn error_response(&self) -> actix_web::HttpResponse {
        HttpResponse::InternalServerError().body(self.err.to_string())
    }
}
impl From<anyhow::Error> for MyError {
    fn from(err: anyhow::Error) -> MyError {
        MyError { err }
    }
}

#[derive(Deserialize)]
pub struct FindEntitiesQueryParams {
    area: Option<String>,
    position: Option<String>,
    radius: Option<f64>,
    name: Option<String>,
    entity_type: Option<String>,
}

// #[get("/findEntities?<area>&<position>&<radius>&<name>&<entity_type>")]
pub async fn find_entities(
    rcon: web::Data<Arc<FactorioRcon>>,
    info: actix_web::web::Query<FindEntitiesQueryParams>,
) -> Result<Json<Vec<FactorioEntity>>, MyError> {
    Ok(Json(
        rcon.find_entities_filtered(
            info.area.clone().map(|area| area.parse().unwrap()),
            info.position
                .clone()
                .map(|position| position.parse().unwrap()),
            info.radius,
            info.name.clone(),
            info.entity_type.clone(),
        )
        .await?,
    ))
}

#[derive(Deserialize)]
pub struct FindTilesQueryParams {
    area: Option<String>,
    position: Option<String>,
    radius: Option<f64>,
    name: Option<String>,
}
// #[get("/findTiles?<area>&<position>&<radius>&<name>")]
pub async fn find_tiles(
    rcon: web::Data<Arc<FactorioRcon>>,
    info: actix_web::web::Query<FindTilesQueryParams>,
) -> Result<Json<Vec<FactorioTile>>, MyError> {
    Ok(Json(
        rcon.find_tiles_filtered(
            info.area.clone().map(|area| area.parse().unwrap()),
            info.position
                .clone()
                .map(|position| position.parse().unwrap()),
            info.radius,
            info.name.clone(),
        )
        .await?,
    ))
}

#[derive(Deserialize)]
pub struct InventoryContentsAtQueryParams {
    query: String,
}
// #[get("/inventoryContentsAt?<query>")]
pub async fn inventory_contents_at(
    rcon: web::Data<Arc<FactorioRcon>>,
    info: actix_web::web::Query<InventoryContentsAtQueryParams>,
) -> Result<Json<Vec<Option<InventoryResponse>>>, MyError> {
    let parts: Vec<&str> = info.query.split(';').collect();
    let entities: Vec<RequestEntity> = parts
        .iter()
        .map(|part| {
            let parts: Vec<&str> = part.split('@').collect();
            RequestEntity {
                name: String::from(parts[0]),
                position: parts[1].parse().unwrap(),
            }
        })
        .collect();
    Ok(Json(rcon.inventory_contents_at(entities).await?))
}

#[derive(Deserialize)]
pub struct MovePlayerQueryParams {
    goal: String,
    radius: Option<f64>,
}
// #[get("/<player_id>/move?<goal>&<radius>")]
pub async fn move_player(
    info: actix_web::web::Query<MovePlayerQueryParams>,
    path: PathInfo<u32>,
    rcon: web::Data<Arc<FactorioRcon>>,
    world: web::Data<Arc<FactorioWorld>>,
) -> Result<Json<FactorioPlayer>, MyError> {
    let player_id = *path;
    let goal: Position = info.goal.parse()?;
    rcon.move_player(&world, player_id, &goal, info.radius)
        .await?;
    let player = world.players.get_one(&player_id);
    match player {
        Some(player) => Ok(Json(player.clone())),
        None => Err(MyError::from(anyhow!("player not found"))),
    }
}

// #[get("/<player_id>/playerInfo")]
pub async fn player_info(
    path: PathInfo<u32>,
    world: web::Data<Arc<FactorioWorld>>,
) -> Result<Json<FactorioPlayer>, MyError> {
    let player_id = *path;

    let player = world.players.get_one(&player_id);
    match player {
        Some(player) => Ok(Json(player.clone())),
        None => Err(MyError::from(anyhow!("player not found"))),
    }
}

#[derive(Deserialize)]
pub struct PlaceEntityQueryParams {
    item: String,
    position: String,
    direction: u8,
}

// #[get("/<player_id>/placeEntity?<item>&<position>&<direction>")]
pub async fn place_entity(
    path: PathInfo<u32>,
    info: actix_web::web::Query<PlaceEntityQueryParams>,
    rcon: web::Data<Arc<FactorioRcon>>,
    world: web::Data<Arc<FactorioWorld>>,
) -> Result<Json<PlaceEntityResult>, MyError> {
    let player_id = *path;
    let entity = rcon
        .place_entity(
            player_id,
            info.item.clone(),
            info.position.parse()?,
            info.direction,
            &world,
        )
        .await?;
    async_std::task::sleep(Duration::from_millis(50)).await;
    let player = world.players.get_one(&player_id);
    match player {
        Some(player) => Ok(Json(PlaceEntityResult {
            entity,
            player: player.clone(),
        })),
        None => Err(MyError::from(anyhow!("player not found"))),
    }
}

#[derive(Deserialize)]
pub struct CheatItemQueryParams {
    name: String,
    count: u32,
}
// #[get("/<player_id>/cheatItem?<name>&<count>")]
#[allow(clippy::too_many_arguments)]
pub async fn cheat_item(
    path: PathInfo<u32>,
    info: actix_web::web::Query<CheatItemQueryParams>,
    world: web::Data<Arc<FactorioWorld>>,
    rcon: web::Data<Arc<FactorioRcon>>,
) -> Result<Json<FactorioPlayer>, MyError> {
    let player_id = *path;
    rcon.cheat_item(player_id, &info.name, info.count).await?;
    async_std::task::sleep(Duration::from_millis(50)).await;
    let player = world.players.get_one(&player_id);
    match player {
        Some(player) => Ok(Json(player.clone())),
        None => Err(MyError::from(anyhow!("player not found"))),
    }
}

#[derive(Deserialize)]
pub struct CheatTechnologyQueryParams {
    tech: String,
}

// #[get("/cheatTechnology?<tech>")]
pub async fn cheat_technology(
    info: actix_web::web::Query<CheatTechnologyQueryParams>,
    rcon: web::Data<Arc<FactorioRcon>>,
) -> Result<Json<Value>, MyError> {
    rcon.cheat_technology(&info.tech).await?;
    Ok(Json(json!({"status": "ok"})))
}

// #[get("/cheatAllTechnologies")]
pub async fn cheat_all_technologies(
    rcon: web::Data<Arc<FactorioRcon>>,
) -> Result<Json<Value>, MyError> {
    rcon.cheat_all_technologies().await?;
    Ok(Json(json!({"status": "ok"})))
}

#[derive(Deserialize)]
pub struct InsertToInventoryQueryParams {
    entity_name: String,
    entity_position: String,
    inventory_type: u32,
    item_name: String,
    item_count: u32,
}
// #[get("/<player_id>/insertToInventory?<entity_name>&<entity_position>&<inventory_type>&<item_name>&<item_count>")]
#[allow(clippy::too_many_arguments)]
pub async fn insert_to_inventory(
    info: actix_web::web::Query<InsertToInventoryQueryParams>,
    path: PathInfo<u32>,
    world: web::Data<Arc<FactorioWorld>>,
    rcon: web::Data<Arc<FactorioRcon>>,
) -> Result<Json<FactorioPlayer>, MyError> {
    let player_id = *path;
    rcon.insert_to_inventory(
        player_id,
        info.entity_name.clone(),
        info.entity_position.parse()?,
        info.inventory_type,
        info.item_name.clone(),
        info.item_count,
        &world,
    )
    .await?;
    async_std::task::sleep(Duration::from_millis(50)).await;
    let player = world.players.get_one(&player_id);
    match player {
        Some(player) => Ok(Json(player.clone())),
        None => Err(MyError::from(anyhow!("player not found"))),
    }
}

#[derive(Deserialize)]
pub struct RemoveFromInventoryQueryParams {
    entity_name: String,
    entity_position: String,
    inventory_type: u32,
    item_name: String,
    item_count: u32,
}

// #[get(
//     "/<player_id>/removeFromInventory?<entity_name>&<entity_position>&<inventory_type>&<item_name>&<item_count>"
// )]
// #[allow(clippy::too_many_arguments)]
pub async fn remove_from_inventory(
    path: PathInfo<u32>,
    info: actix_web::web::Query<RemoveFromInventoryQueryParams>,
    rcon: web::Data<Arc<FactorioRcon>>,
    world: web::Data<Arc<FactorioWorld>>,
) -> Result<Json<FactorioPlayer>, MyError> {
    let player_id = *path;
    rcon.remove_from_inventory(
        player_id,
        info.entity_name.clone(),
        info.entity_position.parse()?,
        info.inventory_type,
        info.item_name.clone(),
        info.item_count,
        &world,
    )
    .await?;
    async_std::task::sleep(Duration::from_millis(50)).await;
    let player = world.players.get_one(&player_id);
    match player {
        Some(player) => Ok(Json(player.clone())),
        None => Err(MyError::from(anyhow!("player not found"))),
    }
}

// #[get("/players")]
pub async fn all_players(
    world: web::Data<Arc<FactorioWorld>>,
) -> Result<Json<Vec<FactorioPlayer>>, MyError> {
    let mut players: Vec<FactorioPlayer> = Vec::new();
    for (_, player) in &world.players.read().unwrap() {
        let player = player.get_one();
        if player.is_some() {
            players.push(player.unwrap().clone());
        }
    }
    Ok(Json(players))
}

// #[get("/itemPrototypes")]
pub async fn item_prototypes(
    world: web::Data<Arc<FactorioWorld>>,
) -> Result<Json<HashMap<String, FactorioItemPrototype>>, MyError> {
    let mut data: HashMap<String, FactorioItemPrototype> = HashMap::new();
    for (key, value) in &world.item_prototypes.read().unwrap() {
        let value = value.get_one().unwrap();
        data.insert(key.clone(), value.clone());
    }
    Ok(Json(data))
}

// #[get("/entityPrototypes")]
pub async fn entity_prototypes(
    world: web::Data<Arc<FactorioWorld>>,
) -> Result<Json<HashMap<String, FactorioEntityPrototype>>, MyError> {
    let mut data: HashMap<String, FactorioEntityPrototype> = HashMap::new();
    for (key, value) in &world.entity_prototypes.read().unwrap() {
        let value = value.get_one().unwrap();
        data.insert(key.clone(), value.clone());
    }
    Ok(Json(data))
}

// #[get("/serverSave")]
pub async fn server_save(rcon: web::Data<Arc<FactorioRcon>>) -> Result<Json<Value>, MyError> {
    rcon.server_save().await?;
    Ok(Json(json!({"status": "ok"})))
}

#[derive(Deserialize)]
pub struct AddResearchQueryParams {
    tech: String,
}
// #[get("/addResearch?<tech>")]
pub async fn add_research(
    info: actix_web::web::Query<AddResearchQueryParams>,
    rcon: web::Data<Arc<FactorioRcon>>,
) -> Result<Json<Value>, MyError> {
    rcon.add_research(&info.tech).await?;
    Ok(Json(json!({"status": "ok"})))
}

#[derive(Deserialize)]
pub struct StoreMapDataQueryParams {
    key: String,
}

// #[post("/storeMapData?<key>", format = "application/json", data = "<value>")]
pub async fn store_map_data(
    rcon: web::Data<Arc<FactorioRcon>>,
    data: Json<Value>,
    info: actix_web::web::Query<StoreMapDataQueryParams>,
) -> Result<Json<Value>, MyError> {
    rcon.store_map_data(&info.key, data.into_inner()).await?;
    Ok(Json(json!({"status": "ok"})))
}
// #[get("/retrieveMapData?<key>")]
pub async fn retrieve_map_data(
    rcon: web::Data<Arc<FactorioRcon>>,
    key: String,
) -> Result<Json<Value>, MyError> {
    let res = rcon.retrieve_map_data(&key).await?;
    match res {
        Some(result) => Ok(Json(result)),
        None => Ok(Json(json!(null))),
    }
}

#[derive(Deserialize)]
pub struct PlaceBlueprintQueryParams {
    blueprint: String,
    position: String,
    direction: Option<u8>,
    force_build: Option<bool>,
    only_ghosts: Option<bool>,
}
// #[get("/<player_id>/placeBlueprint?<position>&<direction>&<force_build>&<blueprint>&<only_ghosts>")]
// #[allow(clippy::too_many_arguments)]
pub async fn place_blueprint(
    world: web::Data<Arc<FactorioWorld>>,
    rcon: web::Data<Arc<FactorioRcon>>,
    path: PathInfo<u32>,
    info: actix_web::web::Query<PlaceBlueprintQueryParams>,
) -> Result<Json<PlaceEntitiesResult>, MyError> {
    let player_id = *path;
    let entities = rcon
        .place_blueprint(
            player_id,
            info.blueprint.clone(),
            &info.position.parse()?,
            info.direction.unwrap_or(0),
            info.force_build.unwrap_or(false),
            info.only_ghosts.unwrap_or(false),
            &world,
        )
        .await?;
    async_std::task::sleep(Duration::from_millis(50)).await;
    let player = world.players.get_one(&player_id);
    match player {
        Some(player) => Ok(Json(PlaceEntitiesResult {
            player: player.clone(),
            entities,
        })),
        None => Err(MyError::from(anyhow!("player not found"))),
    }
}

#[derive(Deserialize)]
pub struct ReviveGhostQueryParams {
    name: String,
    position: String,
}
// #[get("/<player_id>/reviveGhost?<position>&<name>")]
// #[allow(clippy::too_many_arguments)]
pub async fn revive_ghost(
    info: actix_web::web::Query<ReviveGhostQueryParams>,
    path: PathInfo<u32>,
    world: web::Data<Arc<FactorioWorld>>,
    rcon: web::Data<Arc<FactorioRcon>>,
) -> Result<Json<PlaceEntityResult>, MyError> {
    let player_id = *path;
    let entity = rcon
        .revive_ghost(player_id, &info.name, &info.position.parse()?, &world)
        .await?;
    async_std::task::sleep(Duration::from_millis(50)).await;
    let player = world.players.get_one(&player_id);
    match player {
        Some(player) => Ok(Json(PlaceEntityResult {
            player: player.clone(),
            entity,
        })),
        None => Err(MyError::from(anyhow!("player not found"))),
    }
}

#[derive(Deserialize)]
pub struct CheatBlueprintQueryParams {
    blueprint: String,
    position: String,
    direction: Option<u8>,
    force_build: Option<bool>,
}
// #[get("/<player_id>/cheatBlueprint?<position>&<direction>&<force_build>&<blueprint>")]
pub async fn cheat_blueprint(
    world: web::Data<Arc<FactorioWorld>>,
    rcon: web::Data<Arc<FactorioRcon>>,
    info: actix_web::web::Query<CheatBlueprintQueryParams>,
    path: PathInfo<u32>,
) -> Result<Json<PlaceEntitiesResult>, MyError> {
    let player_id = *path;
    let entities = rcon
        .cheat_blueprint(
            player_id,
            info.blueprint.clone(),
            &info.position.parse()?,
            info.direction.unwrap_or(0),
            info.force_build.unwrap_or(false),
        )
        .await?;
    async_std::task::sleep(Duration::from_millis(50)).await;
    let player = world.players.get_one(&player_id);
    match player {
        Some(player) => Ok(Json(PlaceEntitiesResult {
            player: player.clone(),
            entities,
        })),
        None => Err(MyError::from(anyhow!("player not found"))),
    }
}

#[derive(Deserialize)]
pub struct ParseBlueprintQueryParams {
    blueprint: String,
}

// #[get("/parseBlueprint?<blueprint>")]
pub async fn parse_blueprint(
    info: actix_web::web::Query<ParseBlueprintQueryParams>,
) -> Result<Json<Value>, MyError> {
    let decoded =
        BlueprintCodec::decode_string(&info.blueprint).expect("failed to parse blueprint");
    Ok(Json(serde_json::to_value(decoded).unwrap()))
}

// #[get("/recipes")]
pub async fn all_recipes(
    world: web::Data<Arc<FactorioWorld>>,
) -> Result<Json<HashMap<String, FactorioRecipe>>, MyError> {
    let mut map: HashMap<String, FactorioRecipe> = HashMap::new();
    if let Some(recipes) = &world.recipes.read() {
        for (name, recipe) in recipes {
            if let Some(recipe) = recipe.get_one() {
                map.insert(name.to_string(), recipe.clone());
            }
        }
    }
    Ok(Json(map))
}
// #[get("/playerForce")]
pub async fn player_force(
    rcon: web::Data<Arc<FactorioRcon>>,
) -> Result<Json<FactorioForce>, MyError> {
    Ok(Json(rcon.player_force().await?))
}

// #[get("/<player_id>/mine?<name>&<position>&<count>")]

#[derive(Deserialize)]
pub struct MineQueryParams {
    name: String,
    position: String,
    count: u32,
}
pub async fn mine(
    info: actix_web::web::Query<MineQueryParams>,
    path: PathInfo<u32>,
    rcon: web::Data<Arc<FactorioRcon>>,
    world: web::Data<Arc<FactorioWorld>>,
) -> Result<Json<FactorioPlayer>, MyError> {
    let player_id = *path;
    rcon.player_mine(
        &world,
        player_id,
        &info.name,
        &info.position.parse()?,
        info.count,
    )
    .await?;
    async_std::task::sleep(Duration::from_millis(50)).await;
    let player = world.players.get_one(&player_id);
    match player {
        Some(player) => Ok(Json(player.clone())),
        None => Err(MyError::from(anyhow!("player not found"))),
    }
}

// #[get("/<player_id>/craft?<recipe>&<count>")]

#[derive(Deserialize)]
pub struct CraftQueryParams {
    recipe: String,
    count: u32,
}
pub async fn craft(
    info: actix_web::web::Query<CraftQueryParams>,
    path: PathInfo<u32>,
    rcon: web::Data<Arc<FactorioRcon>>,
    world: web::Data<Arc<FactorioWorld>>,
) -> Result<Json<FactorioPlayer>, MyError> {
    let player_id = *path;
    rcon.player_craft(&world, player_id, &info.recipe, info.count)
        .await?;
    async_std::task::sleep(Duration::from_millis(50)).await;
    let player = world.players.get_one(&player_id);
    match player {
        Some(player) => Ok(Json(player.clone())),
        None => Err(MyError::from(anyhow!("player not found"))),
    }
}
