use crate::helpers::octo::{init_octocrab, reply_to_latest_pr, post_markdown_as_comment};
use crate::services::groq::{
  extract_new_functions, json_to_xml, save_xml_to_file, send_request_to_groq,
}; // Import Groq functions
use axum::http::HeaderMap;
use serde_json::Value;
use std::error::Error;

#[derive(Debug)]
pub struct GitHubEvent {
  pub owner: String,
  pub repo: String,
  pub pull_number: u64,
  pub installation_id: u64,
  pub commit_sha: String,
}

pub fn get_installation_id(payload: &Value) -> Option<u64> {
  payload
    .get("installation")
    .and_then(|installation| installation.get("id"))
    .and_then(|id| id.as_u64())
}

/// Process the webhook payload and headers to extract the GitHub event details and trigger Groq processing.
pub async fn process_github_payload(headers: &HeaderMap, payload: &Value) -> GitHubEvent {
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
          commit_sha = payload["after"].as_str().unwrap_or("").to_string();
          if let Some(id) = get_installation_id(payload) {
            installation_id = id;
          } else {
            println!("Installation ID missing in webhook payload");
          }
          println!(
            "Received pull_request.synchronize event for PR #{} in {}/{} with commit SHA {}.",
            pull_number, owner, repo, commit_sha
          );
        }
      }
    } else {
      // Handle other events (e.g., push event)
      owner = payload["repository"]["owner"]["login"].as_str().unwrap_or("").to_string();
      commit_sha = payload["after"].as_str().unwrap_or("").to_string();
      repo = payload["repository"]["name"].as_str().unwrap_or("").to_string();
      println!("Webhook received successfully!");
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
    "Values were successfully obtained. O: {}; R: {}; PR: #{}; IID: {}; SHA: {}",
    owner, repo, pull_number, installation_id, commit_sha
  );

  GitHubEvent { owner, repo, pull_number, installation_id, commit_sha }
}

/// Process the event and swap the installation ID for an installation token.
/// This function creates a JWT and then exchanges it for an installation token.
/// It returns the token as a String.
pub async fn process_event_and_get_token(
  headers: &HeaderMap,
  payload: &Value,
) -> Result<String, Box<dyn Error>> {
  let event = process_github_payload(headers, payload).await;

  if event.installation_id == 0 {
    return Err("No installation ID found in payload".into());
  }

  // Create the JWT using your helper function
  let jwt = crate::helpers::jwt::create_jwt()?;

  // Exchange the JWT for an installation token using your helper function
  let token =
    crate::helpers::jwt::exchange_jwt_for_installation_token(&jwt, event.installation_id).await?;

  // Initialize Octocrab with the installation token.
  let octo = init_octocrab(token.to_string());

  // ✅ **Run the Groq Pipeline for This Commit**
  if !event.commit_sha.is_empty() {
    println!("Extracting new functions from commit: {}", event.commit_sha);

    let _ = extract_new_functions(&event.owner, &event.repo, &event.commit_sha, &octo).await;
    let xml_output = json_to_xml().await;
    save_xml_to_file(&xml_output);

    println!("Sending extracted functions to Groq AI...");
    let groq_response = send_request_to_groq().await;

    match groq_response {
        Ok(response) => {
            println!("Groq AI Analysis Result:\n{}", response);
            // Here, assume your response is a markdown payload.
            if let Err(e) = post_markdown_as_comment(&octo, &event.owner, &event.repo, event.pull_number, &response).await {
                eprintln!("Failed to post markdown comment: {:?}", e);
            }
        }
        Err(e) => eprintln!("❌ Groq AI Request Failed: {:?}", e),
    }
} else {
    println!("⚠️ No commit SHA found, skipping Groq processing.");
}


  // Verify authentication by fetching the current user.
  match octo.current().user().await {
    Ok(user) => {
      println!("Authentication verified! Authenticated as: {:?}", user);
    }
    Err(err) => {
      println!("Failed to verify authentication: {:?}", err);
    }
  }

  // Now automate replying to the latest PR commit with any comment. If it's a change, then reply with a message. Else, reply with mechanic has no comment.
  // test_list_pull_requests(&octo, &event.owner, &event.repo).await;
  // reply_to_latest_pr(&octo, &event.owner, &event.repo).await;

  Ok(token)
}
