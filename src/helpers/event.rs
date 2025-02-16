use axum::http::HeaderMap;
use serde_json::Value;

#[derive(Debug)]
pub struct GitHubEvent {
    pub owner: String,
    pub repo: String,
    pub pull_number: u64,
    pub installation_id: u64,
    pub commit_sha: String,
}

pub fn get_installation_id(payload: &Value) -> Option<u64> {
    payload.get("installation")
           .and_then(|installation| installation.get("id"))
           .and_then(|id| id.as_u64())
}

/// Process the webhook payload and headers to extract the GitHub event details.
pub fn process_github_payload(headers: &HeaderMap, payload: &Value) -> GitHubEvent {
    let mut owner = String::new();
    let mut repo = String::new();
    let mut pull_number = 0;
    let mut installation_id = 0;
    let mut commit_sha = String::new();

    if let Some(event) = headers.get("X-GitHub-Event").and_then(|v| v.to_str().ok()) {
        if event == "pull_request" {
            if let Some(action) = payload.get("action").and_then(|v| v.as_str()) {
                if action == "synchronize" {
                    owner = payload["repository"]["owner"]["login"].as_str().unwrap_or("").to_string();
                    repo = payload["repository"]["name"].as_str().unwrap_or("").to_string();
                    pull_number = payload["pull_request"]["number"].as_u64().unwrap_or(0);
                    if let Some(id) = get_installation_id(payload) {
                        installation_id = id;
                    } else {
                        println!("Installation ID missing in webhook payload");
                    }
                    println!(
                        "Received pull_request.synchronize event for PR #{} in {}/{}",
                        pull_number, owner, repo
                    );
                }
            }
        } else {
            // Fallback for non-pull_request events
            owner = payload["repository"]["owner"]["login"].as_str().unwrap_or("").to_string();
            commit_sha = payload["after"].as_str().unwrap_or("").to_string();
            repo = payload["repository"]["name"].as_str().unwrap_or("").to_string();
            println!("Webhook Received - Owner: {}, Repo: {}, SHA: {}", owner, repo, commit_sha);
            if let Some(id) = get_installation_id(payload) {
                installation_id = id;
            } else {
                println!("Installation ID missing in webhook payload");
            }
        }
    } else {
        println!("X-GitHub-Event header missing.");
    }

    println!(
        "owner: {}, repo: {}, pull_number: {}, installation_id: {}, commit_sha: {}",
        owner, repo, pull_number, installation_id, commit_sha
    );

    GitHubEvent {
        owner,
        repo,
        pull_number,
        installation_id,
        commit_sha,
    }
}
