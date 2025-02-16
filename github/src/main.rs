// main.rs
use axum::{Router, response::IntoResponse, routing::post, Json};
use std::net::SocketAddr;

// This handler remains as in your boilerplate.
async fn github_wh_test_handler(Json(payload): Json<serde_json::Value>) -> impl IntoResponse {
    // println!("Received webhook: {:?}", payload);
    // println!("End of payload");
    let owner = payload["repository"]["owner"]["login"].as_str().unwrap_or("").to_string();
    let commit_sha = payload["after"].as_str().unwrap_or("").to_string();
    let repo_name = payload["repository"]["name"].as_str().unwrap_or("").to_string();

    println!("Found owner: {}, SHA: {}, name: {}", owner, commit_sha, repo_name);
    (owner, commit_sha, repo_name);
}

// Build and serve the Axum app.
pub async fn serve() {
    let app = Router::new()
        // New route to trigger our GitHub event sending.
        // .route("/send-event", post(send_github_event_handler))
        // Your original webhook test route.
        .route("/github-wh-test", post(github_wh_test_handler));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Successfully listening on {}. You can now make requests.", addr);
    axum_server::bind(addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[tokio::main]
async fn main() {
    serve().await;
}
