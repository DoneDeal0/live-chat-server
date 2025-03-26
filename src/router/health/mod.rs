use axum::{Router, routing::get};

mod health;
use health::check_health;

pub fn health_routes() -> Router {
    Router::new().route("/check-health", get(check_health))
}
