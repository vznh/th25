// main.rs
use axum::{Router, response::IntoResponse, routing::post, extract::Json};
use reqwest::Client;
// use serde_json::Value;
// use serde_json::json;
use std::net::SocketAddr;
use std::sync::Arc;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use hex;


pub async fn serve() {
    let client = Arc::new(Client::new());
    let app = Router::new()
        .route("/webhook", post(handle_github_webhook));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!(
        "Successfully listening on {}. You can now make requests.",
        addr
    );
    axum_server::bind(addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[tokio::main]
async fn main() {
    serve().await;
}

// handling github webhook
async fn handle_github_webhook(Json(payload): Json<serde_json::Value>) -> impl IntoResponse {
    println!("Received webhook: {:?}", payload);
    println!("End of payload");
}
