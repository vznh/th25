// main.rs
use axum::{Router, routing::post};
use std::net::SocketAddr;
use github::handlers::webhook::github_wh_test_handler;

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
