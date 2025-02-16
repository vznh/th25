// webhook.rs
use crate::helpers::event::process_event_and_get_token;
use axum::{Json, response::IntoResponse};
use serde_json::Value;

pub async fn github_wh_test_handler(
  headers: axum::http::HeaderMap,
  Json(payload): Json<Value>,
) -> impl IntoResponse {
  match process_event_and_get_token(&headers, &payload).await {
    Ok(token) => {
        println!("Successfully obtained installation token: {}", token);
        token // Return the token as the response
    }
    Err(e) => {
        println!("Error processing event: {}", e);
        format!("Error: {}", e)
    }
}
}
