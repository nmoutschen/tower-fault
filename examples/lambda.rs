use lambda_http::{service_fn, tower::ServiceBuilder, Error, IntoResponse, Request};
use tower_fault_injector::latency::LatencyLayer;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let handler = ServiceBuilder::new()
        // Add a LatencyLayer with a 50% probability of injecting
        // 200 to 500 milliseconds of latency.
        .layer(LatencyLayer::new(0.5, 200..500).unwrap())
        .service(service_fn(my_handler));

    lambda_http::run(handler).await?;
    Ok(())
}

async fn my_handler(_event: Request) -> Result<impl IntoResponse, Error> {
    Ok("Hello, world!")
}
