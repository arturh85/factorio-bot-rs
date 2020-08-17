use std::collections::HashMap;
use std::sync::Arc;

use rocket::response::{Debug, Responder};
use rocket::{response, Request, Response, State};
use rocket_contrib::json::{Json, JsonValue};

use crate::types::{
    FactorioEntityPrototype, FactorioItemPrototype, FactorioPlayer, FactorioRecipe, FactorioResult,
    Position, RequestEntity,
};

use crate::factorio::output_parser::FactorioWorld;
use crate::factorio::rcon::FactorioRcon;
use factorio_blueprint::BlueprintCodec;
use rocket::http::{ContentType, Status};
use serde::Serialize;
use serde_json::Value;
use tokio::time::Duration;

#[derive(Debug)]
pub struct ApiResponse {
    json: JsonValue,
    status: Status,
}
impl<'r> Responder<'r, 'r> for ApiResponse {
    fn respond_to(self, req: &Request) -> response::Result<'r> {
        Response::build_from(self.json.respond_to(&req).unwrap())
            .status(self.status)
            .header(ContentType::JSON)
            .ok()
    }
}

#[get("/findEntities?<area>&<position>&<radius>&<name>&<entity_type>")]
pub async fn find_entities(
    area: Option<String>,
    position: Option<String>,
    radius: Option<f64>,
    name: Option<String>,
    entity_type: Option<String>,
    rcon: State<'_, Arc<FactorioRcon>>,
) -> Result<ApiResponse, Debug<anyhow::Error>> {
    response_from_serialize_result(
        rcon.find_entities_filtered(
            area.map(|area| area.parse().unwrap()),
            position.map(|position| position.parse().unwrap()),
            radius,
            name,
            entity_type,
        )
        .await,
    )
}

#[get("/findTiles?<area>&<position>&<radius>&<name>")]
pub async fn find_tiles(
    area: Option<String>,
    position: Option<String>,
    radius: Option<f64>,
    name: Option<String>,
    rcon: State<'_, Arc<FactorioRcon>>,
) -> Result<ApiResponse, Debug<anyhow::Error>> {
    response_from_serialize_result(
        rcon.find_tiles_filtered(
            area.map(|area| area.parse().unwrap()),
            position.map(|position| position.parse().unwrap()),
            radius,
            name,
        )
        .await,
    )
}

#[get("/inventoryContentsAt?<query>")]
pub async fn inventory_contents_at(
    query: String,
    rcon: State<'_, Arc<FactorioRcon>>,
) -> Result<ApiResponse, Debug<anyhow::Error>> {
    let parts: Vec<&str> = query.split(';').collect();
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
    response_from_serialize_result(rcon.inventory_contents_at(entities).await)
}

#[get("/<player_id>/move?<goal>&<radius>")]
pub async fn move_player(
    player_id: u32,
    goal: String,
    radius: Option<f64>,
    rcon: State<'_, Arc<FactorioRcon>>,
    world: State<'_, Arc<FactorioWorld>>,
) -> Result<ApiResponse, Debug<anyhow::Error>> {
    let goal: Position = goal.parse()?;
    response_from_result(
        Some(player_id),
        &world,
        rcon.move_player(&world, player_id, &goal, radius).await,
    )
}

#[get("/<player_id>/playerInfo")]
pub async fn player_info(
    player_id: u32,
    world: State<'_, Arc<FactorioWorld>>,
) -> Result<ApiResponse, Debug<anyhow::Error>> {
    let player = world.players.get_one(&player_id).unwrap();
    Ok(ApiResponse {
        json: JsonValue::from(serde_json::to_value(&*player).unwrap()),
        status: Status::Ok,
    })
}

#[get("/<player_id>/placeEntity?<item>&<position>&<direction>")]
pub async fn place_entity(
    player_id: u32,
    item: String,
    position: String,
    direction: u8,
    rcon: State<'_, Arc<FactorioRcon>>,
    world: State<'_, Arc<FactorioWorld>>,
) -> Result<ApiResponse, Debug<anyhow::Error>> {
    let position: Position = position.parse()?;
    match rcon
        .place_entity(player_id, item, position, direction, &world)
        .await
    {
        Ok(entity) => {
            async_std::task::sleep(Duration::from_millis(50)).await;
            let player =
                serde_json::to_value(world.players.get_one(&player_id).unwrap().clone()).unwrap();
            let entity = serde_json::to_value(entity).unwrap();
            Ok(ApiResponse {
                json: json!({"player": player, "entity": entity}),
                status: Status::Ok,
            })
        }
        Err(err) => Ok(ApiResponse {
            json: json!({ "error": err.to_string() }),
            status: Status::InternalServerError,
        }),
    }
}

#[get("/<player_id>/cheatItem?<name>&<count>")]
#[allow(clippy::too_many_arguments)]
pub async fn cheat_item(
    player_id: u32,
    name: String,
    count: u32,
    world: State<'_, Arc<FactorioWorld>>,
    rcon: State<'_, Arc<FactorioRcon>>,
) -> Result<ApiResponse, Debug<anyhow::Error>> {
    let result = rcon.cheat_item(player_id, &name, count).await;
    if result.is_ok() {
        // wait for inventory update event
        // loop {
        //     let changed_player_id = world.rx_player_inventory_changed.recv().unwrap();
        //     if player_id == changed_player_id {
        //         break;
        //     }
        // }
        async_std::task::sleep(Duration::from_millis(50)).await;
    }
    response_from_result(Some(player_id), &world, result)
}

#[get("/cheatTechnology?<tech>")]
#[allow(clippy::too_many_arguments)]
pub async fn cheat_technology(
    tech: String,
    world: State<'_, Arc<FactorioWorld>>,
    rcon: State<'_, Arc<FactorioRcon>>,
) -> Result<ApiResponse, Debug<anyhow::Error>> {
    response_from_result(None, &world, rcon.cheat_technology(&tech).await)
}

#[get("/cheatAllTechnologies")]
#[allow(clippy::too_many_arguments)]
pub async fn cheat_all_technologies(
    world: State<'_, Arc<FactorioWorld>>,
    rcon: State<'_, Arc<FactorioRcon>>,
) -> Result<ApiResponse, Debug<anyhow::Error>> {
    response_from_result(None, &world, rcon.cheat_all_technologies().await)
}

#[get("/<player_id>/insertToInventory?<entity_name>&<entity_position>&<inventory_type>&<item_name>&<item_count>")]
#[allow(clippy::too_many_arguments)]
pub async fn insert_to_inventory(
    player_id: u32,
    entity_name: String,
    entity_position: String,
    inventory_type: u32,
    item_name: String,
    item_count: u32,
    world: State<'_, Arc<FactorioWorld>>,
    rcon: State<'_, Arc<FactorioRcon>>,
) -> Result<ApiResponse, Debug<anyhow::Error>> {
    let entity_position: Position = entity_position.parse()?;
    let result = rcon
        .insert_to_inventory(
            player_id,
            entity_name,
            entity_position,
            inventory_type,
            item_name,
            item_count,
            &world,
        )
        .await;
    if result.is_ok() {
        // wait for inventory update event
        // loop {
        //     let changed_player_id = world.rx_player_inventory_changed.recv().unwrap();
        //     if player_id == changed_player_id {
        //         break;
        //     }
        // }
        async_std::task::sleep(Duration::from_millis(50)).await;
    }
    response_from_result(Some(player_id), &world, result)
}

#[get(
    "/<player_id>/removeFromInventory?<entity_name>&<entity_position>&<inventory_type>&<item_name>&<item_count>"
)]
#[allow(clippy::too_many_arguments)]
pub async fn remove_from_inventory(
    player_id: u32,
    entity_name: String,
    entity_position: String,
    inventory_type: u32,
    item_name: String,
    item_count: u32,
    rcon: State<'_, Arc<FactorioRcon>>,
    world: State<'_, Arc<FactorioWorld>>,
) -> Result<ApiResponse, Debug<anyhow::Error>> {
    let entity_position: Position = entity_position.parse()?;
    let result = rcon
        .remove_from_inventory(
            player_id,
            entity_name,
            entity_position,
            inventory_type,
            item_name,
            item_count,
            &world,
        )
        .await;
    if result.is_ok() {
        // wait for inventory update event
        // loop {
        //     let changed_player_id = world.rx_player_inventory_changed.recv().unwrap();
        //     if player_id == changed_player_id {
        //         break;
        //     }
        // }
        async_std::task::sleep(Duration::from_millis(50)).await;
    }
    response_from_result(Some(player_id), &world, result)
}

#[get("/players")]
pub async fn all_players(
    world: State<'_, Arc<FactorioWorld>>,
) -> Result<ApiResponse, Debug<anyhow::Error>> {
    let mut players: Vec<FactorioPlayer> = Vec::new();
    for player_id in 0..8u32 {
        let player = world.players.get_one(&player_id);
        if player.is_some() {
            players.push(player.unwrap().clone());
        }
    }

    Ok(ApiResponse {
        json: JsonValue::from(serde_json::to_value(players).unwrap()),
        status: Status::Ok,
    })
}

#[get("/itemPrototypes")]
pub async fn item_prototypes(
    world: State<'_, Arc<FactorioWorld>>,
) -> Result<ApiResponse, Debug<anyhow::Error>> {
    let mut data: HashMap<String, FactorioItemPrototype> = HashMap::new();
    for (key, value) in &world.item_prototypes.read().unwrap() {
        let value = value.get_one().unwrap();
        data.insert(key.clone(), value.clone());
    }
    Ok(ApiResponse {
        json: JsonValue::from(serde_json::to_value(data).unwrap()),
        status: Status::Ok,
    })
}

#[get("/entityPrototypes")]
pub async fn entity_prototypes(
    world: State<'_, Arc<FactorioWorld>>,
) -> Result<ApiResponse, Debug<anyhow::Error>> {
    let mut data: HashMap<String, FactorioEntityPrototype> = HashMap::new();
    for (key, value) in &world.entity_prototypes.read().unwrap() {
        let value = value.get_one().unwrap();
        data.insert(key.clone(), value.clone());
    }
    Ok(ApiResponse {
        json: JsonValue::from(serde_json::to_value(data).unwrap()),
        status: Status::Ok,
    })
}

#[get("/serverSave")]
pub async fn server_save(
    world: State<'_, Arc<FactorioWorld>>,
    rcon: State<'_, Arc<FactorioRcon>>,
) -> Result<ApiResponse, Debug<anyhow::Error>> {
    response_from_result(None, &world, rcon.server_save().await)
}

#[get("/addResearch?<tech>")]
pub async fn add_research(
    world: State<'_, Arc<FactorioWorld>>,
    rcon: State<'_, Arc<FactorioRcon>>,
    tech: String,
) -> Result<ApiResponse, Debug<anyhow::Error>> {
    response_from_result(None, &world, rcon.add_research(&tech).await)
}

#[post("/storeMapData?<key>", format = "application/json", data = "<value>")]
pub async fn store_map_data(
    world: State<'_, Arc<FactorioWorld>>,
    rcon: State<'_, Arc<FactorioRcon>>,
    key: String,
    value: Json<Value>,
) -> Result<ApiResponse, Debug<anyhow::Error>> {
    response_from_result(
        None,
        &world,
        rcon.store_map_data(&key, value.into_inner()).await,
    )
}
#[get("/retrieveMapData?<key>")]
pub async fn retrieve_map_data(
    rcon: State<'_, Arc<FactorioRcon>>,
    key: String,
) -> Result<ApiResponse, Debug<anyhow::Error>> {
    let res = rcon.retrieve_map_data(&key).await?;

    match res {
        Some(result) => Ok(ApiResponse {
            json: JsonValue::from(result),
            status: Status::Ok,
        }),
        None => Ok(ApiResponse {
            json: json!(null),
            status: Status::Ok,
        }),
    }
}

#[get("/<player_id>/placeBlueprint?<position>&<direction>&<force_build>&<blueprint>&<only_ghosts>")]
#[allow(clippy::too_many_arguments)]
pub async fn place_blueprint(
    world: State<'_, Arc<FactorioWorld>>,
    rcon: State<'_, Arc<FactorioRcon>>,
    player_id: u32,
    blueprint: String,
    position: String,
    direction: Option<u8>,
    force_build: Option<bool>,
    only_ghosts: Option<bool>,
) -> Result<ApiResponse, Debug<anyhow::Error>> {
    let position = position.parse().unwrap();
    let result = rcon
        .place_blueprint(
            player_id,
            blueprint,
            &position,
            direction.unwrap_or(0),
            force_build.unwrap_or(false),
            only_ghosts.unwrap_or(false),
            &world,
        )
        .await;

    if result.is_ok() {
        // wait for inventory update event
        // loop {
        //     let changed_player_id = world.rx_player_inventory_changed.recv().unwrap();
        //     if player_id == changed_player_id {
        //         break;
        //     }
        // }
        async_std::task::sleep(Duration::from_millis(50)).await;
    }
    let entities = result.unwrap();
    let player = serde_json::to_value(world.players.get_one(&player_id).unwrap().clone()).unwrap();
    let entities = serde_json::to_value(entities).unwrap();
    Ok(ApiResponse {
        json: json!({"player": player, "entities": entities}),
        status: Status::Ok,
    })
}

#[get("/<player_id>/cheatBlueprint?<position>&<direction>&<force_build>&<blueprint>")]
pub async fn cheat_blueprint(
    world: State<'_, Arc<FactorioWorld>>,
    rcon: State<'_, Arc<FactorioRcon>>,
    player_id: u32,
    blueprint: String,
    position: String,
    direction: Option<u8>,
    force_build: Option<bool>,
) -> Result<ApiResponse, Debug<anyhow::Error>> {
    let position = position.parse().unwrap();
    let result = rcon
        .cheat_blueprint(
            player_id,
            blueprint,
            &position,
            direction.unwrap_or(0),
            force_build.unwrap_or(false),
        )
        .await;

    if result.is_ok() {
        // wait for inventory update event
        // loop {
        //     let changed_player_id = world.rx_player_inventory_changed.recv().unwrap();
        //     if player_id == changed_player_id {
        //         break;
        //     }
        // }
        async_std::task::sleep(Duration::from_millis(50)).await;
    }
    let entities = result.unwrap();
    let player = serde_json::to_value(world.players.get_one(&player_id).unwrap().clone()).unwrap();
    let entities = serde_json::to_value(entities).unwrap();
    Ok(ApiResponse {
        json: json!({"player": player, "entities": entities}),
        status: Status::Ok,
    })
}

#[get("/parseBlueprint?<blueprint>")]
pub async fn parse_blueprint(blueprint: String) -> Result<ApiResponse, Debug<anyhow::Error>> {
    let decoded = BlueprintCodec::decode_string(&blueprint).expect("failed to parse blueprint");
    Ok(ApiResponse {
        json: JsonValue::from(serde_json::to_value(decoded).unwrap()),
        status: Status::Ok,
    })
}

#[get("/recipes")]
pub async fn all_recipes(
    world: State<'_, Arc<FactorioWorld>>,
) -> Result<ApiResponse, Debug<anyhow::Error>> {
    let mut map: HashMap<String, FactorioRecipe> = HashMap::new();
    if let Some(recipes) = &world.recipes.read() {
        for (name, recipe) in recipes {
            if let Some(recipe) = recipe.get_one() {
                map.insert(name.to_string(), recipe.clone());
            }
        }
    }
    Ok(ApiResponse {
        json: JsonValue::from(serde_json::to_value(map).unwrap()),
        status: Status::Ok,
    })
}
#[get("/playerForce")]
pub async fn player_force(
    rcon: State<'_, Arc<FactorioRcon>>,
) -> Result<ApiResponse, Debug<anyhow::Error>> {
    let result = rcon.player_force().await?;
    Ok(ApiResponse {
        json: JsonValue::from(serde_json::to_value(result).unwrap()),
        status: Status::Ok,
    })
}

#[get("/<player_id>/mine?<name>&<position>&<count>")]
pub async fn mine(
    player_id: u32,
    name: String,
    position: String,
    count: u32,
    rcon: State<'_, Arc<FactorioRcon>>,
    world: State<'_, Arc<FactorioWorld>>,
) -> Result<ApiResponse, Debug<anyhow::Error>> {
    let position: Position = position.parse()?;
    let result = rcon
        .player_mine(&world, player_id, &name, &position, count)
        .await;

    if result.is_ok() {
        // wait for inventory update event
        // loop {
        //     let changed_player_id = world.rx_player_inventory_changed.recv().unwrap();
        //     if player_id == changed_player_id {
        //         break;
        //     }
        // }
        async_std::task::sleep(Duration::from_millis(50)).await;
    }
    response_from_result(Some(player_id), &world, result)
}

#[get("/<player_id>/craft?<recipe>&<count>")]
pub async fn craft(
    player_id: u32,
    recipe: String,
    count: u32,
    rcon: State<'_, Arc<FactorioRcon>>,
    world: State<'_, Arc<FactorioWorld>>,
) -> Result<ApiResponse, Debug<anyhow::Error>> {
    let result = rcon.player_craft(&world, player_id, &recipe, count).await;
    if result.is_ok() {
        // wait for inventory update event
        // loop {
        //     let changed_player_id = world.rx_player_inventory_changed.recv().unwrap();
        //     if player_id == changed_player_id {
        //         break;
        //     }
        // }
        async_std::task::sleep(Duration::from_millis(50)).await;
    }
    response_from_result(Some(player_id), &world, result)
}

pub fn response_from_result(
    player_id: Option<u32>,
    world: &State<'_, Arc<FactorioWorld>>,
    result: Result<(), anyhow::Error>,
) -> Result<ApiResponse, Debug<anyhow::Error>> {
    match result {
        Ok(_) => {
            let response = match player_id {
                Some(player_id) => {
                    serde_json::to_value(world.players.get_one(&player_id).unwrap().clone())
                }
                None => serde_json::to_value(FactorioResult {
                    success: true,
                    output: vec![],
                }),
            };
            Ok(ApiResponse {
                json: JsonValue::from(response.unwrap()),
                status: Status::Ok,
            })
        }
        Err(err) => {
            let response = FactorioResult {
                success: false,
                output: vec![err.to_string()],
            };
            return Ok(ApiResponse {
                json: JsonValue::from(serde_json::to_value(response).unwrap()),
                status: Status::InternalServerError,
            });
        }
    }
}

pub fn response_from_serialize_result<T>(
    result: Result<T, anyhow::Error>,
) -> Result<ApiResponse, Debug<anyhow::Error>>
where
    T: Serialize,
{
    match result {
        Ok(response) => Ok(ApiResponse {
            json: JsonValue::from(serde_json::to_value(response).unwrap()),
            status: Status::Ok,
        }),
        Err(err) => {
            let response = FactorioResult {
                success: false,
                output: vec![err.to_string()],
            };
            return Ok(ApiResponse {
                json: JsonValue::from(serde_json::to_value(response).unwrap()),
                status: Status::InternalServerError,
            });
        }
    }
}
