use axum::{Router, routing::get};

mod check_health;
use check_health::check_health;

pub fn health_routes() -> Router {
    Router::new().route("/check-health", get(check_health))
}
