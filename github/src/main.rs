// main.rs
use axum::Router;
use axum::response::IntoResponse;
use axum::routing::post;
use reqwest::Client;
use serde_json::Value;
use std::net::SocketAddr;
use std::sync::Arc;

pub async fn serve() {
    let client = Arc::new(Client::new());
    let app = Router::new()
        .route("/github-wh-test", post(github_wh_test_handler));

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

// handlers

// example that takes in POST requests
async fn github_wh_test_handler(req: axum::http::Request<axum::body::Body>) -> impl IntoResponse {
    println!("Request received: {:?}", req);
    let response = "Request content";
    println!("Request finished");
    response
}
