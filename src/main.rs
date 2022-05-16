mod custom_ws;
mod physics_engine;
mod state;

use state::State;

use actix::Actor;
use actix_files as fs;
use actix_files::NamedFile;
use actix_web::{get, web, App, Error, HttpRequest, HttpServer, Result};
use std::path::PathBuf;

/// Handles HTTP requests for files
/// Looks in the /static/ directory for file requested
#[get("/{filename:.*}")]
async fn index(req: HttpRequest) -> Result<fs::NamedFile, Error> {
    let path: std::path::PathBuf = req.match_info().query("filename").parse().unwrap();
    let file = fs::NamedFile::open_async(PathBuf::from("./static/").join(path)).await?;
    Ok(file.use_last_modified(true))
}

/// Reroute home page to index.html
#[get("/")]
async fn default_page() -> Result<fs::NamedFile, Error> {
    Ok(NamedFile::open_async("./static/index.html").await?
        .use_last_modified(true))
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
            .service(default_page)
            .service(index)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
