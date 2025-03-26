mod middleware;
mod models;
mod router;
use axum::Router;
use dotenvy::dotenv;
use middleware::{compression::compress_responses, cors::get_cors};
use router::{chat::chat_routes, health::health_routes};
use std::{env, error::Error};
use tokio::net::TcpListener;
use tracing::{Level, error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_max_level(Level::ERROR)
        .with_writer(std::io::stderr)
        .init();

    dotenv().ok();

    let addr = env::var("PORT").unwrap_or_else(|e| {
        error!("Failed to read PORT: {}, defaulting to 0.0.0.8080", e);
        "0.0.0.0:8080".to_string()
    });

    let app = Router::new()
        .merge(health_routes())
        .nest("/chat", chat_routes())
        .layer(get_cors())
        .layer(compress_responses());

    let listener = TcpListener::bind(&addr).await.map_err(|e| {
        error!("Failed to bind to {}: {}", addr, e);
        e
    })?;

    axum::serve(listener, app).await.map_err(|e| {
        error!("Server failed: {}", e);
        e
    })?;

    Ok(())
}
