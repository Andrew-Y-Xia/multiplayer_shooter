mod custom_ws;
mod physics_engine;
mod state;

use state::State;

use actix::Actor;
use actix_files::NamedFile;
use actix_web::{web, App, HttpRequest, HttpServer, Result};
use std::path::PathBuf;

/// Handles HTTP requests for files
/// Looks in the /www/ directory for file requested
async fn index(req: HttpRequest) -> Result<NamedFile> {
    let path: PathBuf = req.match_info().query("filename").parse().unwrap();
    Ok(NamedFile::open(PathBuf::from("./www/").join(path))?)
}

/// Reroute home page to index.html
async fn default_page(_req: HttpRequest) -> Result<NamedFile> {
    Ok(NamedFile::open(PathBuf::from("./www/index.html"))?)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Start physics engine actor
    let physics_actor = physics_engine::PhysicsEngine::new();
    let physics_engine_address = physics_actor.start();

    // Construct app state
    let app_state = web::Data::new(State::new(physics_engine_address));

    // Initialize and run server
    HttpServer::new(move || {
        App::new()
            // Gives app a pointer to the app state
            // app_state can be accessed through the functions registered below
            .app_data(app_state.clone())
            // Routes Websocket connections
            .route("/ws/", web::get().to(custom_ws::index_ws))
            // Routes file requests
            .route("/", web::get().to(default_page))
            .route("/{filename:.*}", web::get().to(index))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
