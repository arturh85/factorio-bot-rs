#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate evmap_derive;
#[macro_use]
extern crate paris;
#[macro_use]
extern crate enum_primitive_derive;
extern crate num_traits;

pub mod factorio;
pub mod types;
pub mod web;

use crate::factorio::output_parser::FactorioWorld;
use crate::factorio::rcon::FactorioRcon;
use config::Config;
use rocket_contrib::json::JsonValue;
use rocket_contrib::serve::StaticFiles;
use std::path::Path;
use std::sync::Arc;

#[catch(404)]
fn not_found() -> JsonValue {
    json!({
        "status": "error",
        "reason": "Resource was not found."
    })
}

#[get("/status")]
pub fn status() -> String {
    String::from("ok")
}

pub async fn build_rocket(
    settings: Config,
    rcon: FactorioRcon,
    world: Arc<FactorioWorld>,
) -> rocket::Rocket {
    let frontend_path = match Path::new("public/").exists() {
        true => "public/",
        false => "frontend/dist/",
    };

    // dotenv().ok();
    info!("🚀 Webserver launched at http://localhost:7123/");
    rocket::ignite()
        // .mount(
        //     "/workspace",
        //     StaticFiles::from(crate_relative!("../workspace/")).rank(9),
        // )
        .mount("/", StaticFiles::from(frontend_path).rank(10))
        .mount("/", routes![status])
        .mount(
            "/api",
            routes![
                web::map_tiles::map_tiles,
                web::rest_api::all_recipes,
                web::rest_api::player_info,
                web::rest_api::move_player,
                web::rest_api::find_entities,
                web::rest_api::mine,
                web::rest_api::craft,
                web::rest_api::all_players,
                web::rest_api::place_entity,
                web::rest_api::insert_to_inventory,
                web::rest_api::remove_from_inventory,
                web::rest_api::server_save,
                web::rest_api::player_force,
                web::rest_api::place_blueprint,
                web::rest_api::item_prototypes,
                web::rest_api::entity_prototypes,
                web::rest_api::parse_blueprint,
                web::rest_api::find_tiles,
                web::rest_api::cheat_blueprint,
                web::rest_api::cheat_item,
                web::rest_api::cheat_technology,
                web::rest_api::cheat_all_technologies,
                web::rest_api::add_research,
                web::rest_api::store_map_data,
                web::rest_api::retrieve_map_data,
                web::rest_api::inventory_contents_at,
            ],
        )
        .manage(world)
        .manage(Arc::new(rcon))
        .manage(settings)
        .register(catchers![not_found])
}
