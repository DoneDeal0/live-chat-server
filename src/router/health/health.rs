use axum::{Json, http::StatusCode};
use serde_json::{Value, json};

// #[debug_handler]
pub async fn check_health() -> (StatusCode, Json<Value>) {
    (
        StatusCode::OK,
        Json(json!({"data": "The server is up and running!"})),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{Router, routing::get};
    use axum_test::TestServer;

    #[tokio::test]
    async fn test_check_health() {
        let app = Router::new().route("/check-health", get(check_health));
        let server = TestServer::new(app).unwrap();

        let response = server.get("/check-health").await;
        response.assert_status(StatusCode::OK);
        response.assert_json(&json!({"data": "The server is up and running!"}));
    }
}
