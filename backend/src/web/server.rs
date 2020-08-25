use crate::factorio::output_parser::FactorioWorld;
use crate::factorio::rcon::FactorioRcon;
use std::path::Path;
use std::sync::Arc;

use crate::ws::session::WsChatSession;
use actix_cors::Cors;
use actix_files as fs;
use actix_web::{
    http, middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder,
};
use actix_web_actors::ws;
use std::env;

async fn status() -> impl Responder {
    HttpResponse::Ok().body("ok")
}

async fn ws_index(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    ws::start(WsChatSession::default(), &req, stream)
}

pub async fn start_webserver(rcon: FactorioRcon, open_browser: bool, world: Arc<FactorioWorld>) {
    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| "7123".into())
        .parse()
        .expect("invalid PORT env");

    let url = format!("http://localhost:{}", port);
    info!("🚀 Webserver ready at: <yellow><underline>{}", url);
    if open_browser {
        webbrowser::open(&url).expect("failed to open browser");
    }
    let frontend_path = match Path::new("public/").exists() {
        true => "public/",
        false => "frontend/dist/",
    };
    let rcon = Arc::new(rcon);
    HttpServer::new(move || {
        App::new()
            .data(world.clone())
            .data(rcon.clone())
            .wrap(
                Cors::new()
                    .allowed_methods(vec!["GET", "POST"])
                    .allowed_header(http::header::ACCEPT)
                    .allowed_header(http::header::CONTENT_TYPE)
                    .allowed_header(http::header::CACHE_CONTROL)
                    .finish(),
            )
            .wrap(middleware::Logger::default())
            // websocket route
            .service(web::resource("/ws/").route(web::get().to(ws_index)))
            .service(
                web::resource("/api/tiles/{tile_z}/{tile_x}/{tile_y}/tile.png")
                    .route(web::get().to(crate::web::map_tiles::map_tiles)),
            )
            .service(
                web::resource("/api/findEntities")
                    .route(web::get().to(crate::web::rest_api::find_entities)),
            )
            .service(
                web::resource("/api/findTiles")
                    .route(web::get().to(crate::web::rest_api::find_tiles)),
            )
            .service(
                web::resource("/api/inventoryContentsAt")
                    .route(web::get().to(crate::web::rest_api::inventory_contents_at)),
            )
            .service(
                web::resource("/api/{player_id}/move")
                    .route(web::get().to(crate::web::rest_api::move_player)),
            )
            .service(
                web::resource("/api/{player_id}/playerInfo")
                    .route(web::get().to(crate::web::rest_api::player_info)),
            )
            .service(
                web::resource("/api/{player_id}/placeEntity")
                    .route(web::get().to(crate::web::rest_api::place_entity)),
            )
            .service(
                web::resource("/api/{player_id}/cheatItem")
                    .route(web::get().to(crate::web::rest_api::cheat_item)),
            )
            .service(
                web::resource("/api/cheatTechnology")
                    .route(web::get().to(crate::web::rest_api::cheat_technology)),
            )
            .service(
                web::resource("/api/cheatAllTechnologies")
                    .route(web::get().to(crate::web::rest_api::cheat_all_technologies)),
            )
            .service(
                web::resource("/api/{player_id}/insertToInventory")
                    .route(web::get().to(crate::web::rest_api::insert_to_inventory)),
            )
            .service(
                web::resource("/api/{player_id}/removeFromInventory")
                    .route(web::get().to(crate::web::rest_api::remove_from_inventory)),
            )
            .service(
                web::resource("/api/players")
                    .route(web::get().to(crate::web::rest_api::all_players)),
            )
            .service(
                web::resource("/api/itemPrototypes")
                    .route(web::get().to(crate::web::rest_api::item_prototypes)),
            )
            .service(
                web::resource("/api/entityPrototypes")
                    .route(web::get().to(crate::web::rest_api::entity_prototypes)),
            )
            .service(
                web::resource("/api/serverSave")
                    .route(web::get().to(crate::web::rest_api::server_save)),
            )
            .service(
                web::resource("/api/addResearch")
                    .route(web::get().to(crate::web::rest_api::add_research)),
            )
            .service(
                web::resource("/api/storeMapData")
                    .route(web::post().to(crate::web::rest_api::store_map_data)),
            )
            .service(
                web::resource("/api/retrieveMapData")
                    .route(web::get().to(crate::web::rest_api::retrieve_map_data)),
            )
            .service(
                web::resource("/api/{player_id}/placeBlueprint")
                    .route(web::get().to(crate::web::rest_api::place_blueprint)),
            )
            .service(
                web::resource("/api/{player_id}/reviveGhost")
                    .route(web::get().to(crate::web::rest_api::revive_ghost)),
            )
            .service(
                web::resource("/api/{player_id}/cheatBlueprint")
                    .route(web::get().to(crate::web::rest_api::cheat_blueprint)),
            )
            .service(
                web::resource("/api/parseBlueprint")
                    .route(web::get().to(crate::web::rest_api::parse_blueprint)),
            )
            .service(
                web::resource("/api/recipes")
                    .route(web::get().to(crate::web::rest_api::all_recipes)),
            )
            .service(
                web::resource("/api/playerForce")
                    .route(web::get().to(crate::web::rest_api::player_force)),
            )
            .service(
                web::resource("/api/{player_id}/mine")
                    .route(web::get().to(crate::web::rest_api::mine)),
            )
            .service(
                web::resource("/api/{player_id}/craft")
                    .route(web::get().to(crate::web::rest_api::craft)),
            )
            .service(
                web::resource("/api/{player_id}/craft")
                    .route(web::get().to(crate::web::rest_api::craft)),
            )
            // .service(crate::web::resource("/graphiql").route(web::get().to(graphiql)))
            .service(web::resource("/status").route(web::get().to(status)))
            // .service(crate::web::resource("/playground").route(web::get().to(playground)))
            // .service(crate::web::resource("/types.d.ts").route(web::get().to(type_d_ts)))
            // .service(crate::web::resource("/schema.graphql").route(web::get().to(schema_graphql)))
            .service(fs::Files::new("/", frontend_path).index_file("index.html"))
    })
    .bind(format!("0.0.0.0:{}", port))
    .expect("failed to bind")
    .run()
    .await
    .unwrap();
}