// webhook.rs
use axum::{Json, response::IntoResponse};

pub async fn github_wh_test_handler(Json(payload): Json<serde_json::Value>) -> impl IntoResponse {
  // println!("Received webhook: {:?}", payload);
  // println!("End of payload");
  let owner = payload["repository"]["owner"]["login"].as_str().unwrap_or("").to_string();
  let commit_sha = payload["after"].as_str().unwrap_or("").to_string();
  let repo_name = payload["repository"]["name"].as_str().unwrap_or("").to_string();

  println!("Found owner: {}, SHA: {}, name: {}", owner, commit_sha, repo_name);
  (owner, commit_sha, repo_name);
}
