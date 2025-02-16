// webhook.rs
use axum::{Json, response::IntoResponse};
use octocrab::Octocrab;
use serde_json::Value;
use std::env;

pub async fn github_wh_test_handler(
  headers: axum::http::HeaderMap,
  Json(payload): Json<Value>,
) -> impl IntoResponse {
  // Check for GitHub event header
  if let Some(event) = headers.get("X-GitHub-Event").and_then(|v| v.to_str().ok()) {
    if event == "pull_request" {
      if let Some(action) = payload.get("action").and_then(|v| v.as_str()) {
        if action == "synchronize" {
          // This is a pull_request.synchronize event (new commits pushed to a PR)
          let owner = payload["repository"]["owner"]["login"].as_str().unwrap_or("").to_string();
          let repo = payload["repository"]["name"].as_str().unwrap_or("").to_string();
          let pull_number = payload["pull_request"]["number"].as_u64().unwrap_or(0);
          println!(
            "Received pull_request.synchronize event for PR #{} in {}/{}",
            pull_number, owner, repo
          );
        }
      }
    }
  }

  // Fallback: For non-pull_request.synchronize events (or push events)
  let owner = payload["repository"]["owner"]["login"].as_str().unwrap_or("").to_string();
  let commit_sha = payload["after"].as_str().unwrap_or("").to_string();
  let repo_name = payload["repository"]["name"].as_str().unwrap_or("").to_string();
  println!("Webhook Received - Owner: {}, Repo: {}, SHA: {}", owner, repo_name, commit_sha);

  // Optionally, call your existing service (e.g. get_changed_files) if needed.
  // let changed_files = get_changed_files(&owner, &repo_name, &commit_sha).await;
  // println!("Changed Files: {:?}", changed_files);
}
