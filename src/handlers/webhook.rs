// webhook.rs
use crate::helpers::event::process_github_payload;
use axum::{Json, response::IntoResponse};
use serde_json::Value;

pub async fn github_wh_test_handler(
  headers: axum::http::HeaderMap,
  Json(payload): Json<Value>,
) -> impl IntoResponse {
  let event = process_github_payload(&headers, &payload);
  println!("GitHub object: {:#?}", event);
}
