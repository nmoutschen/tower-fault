use axum::{routing::get, Router, Server};
use tower_fault::latency::LatencyLayer;

// Simple service that returns a string.
async fn handler() -> &'static str {
    "Hello, world!"
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(handler))
        // Add a LatencyLayer with a 50% probability of injecting
        // 200 to 500 milliseconds of latency.
        .layer(LatencyLayer::new(0.5, 200..500));

    // Start the axum server.
    Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
