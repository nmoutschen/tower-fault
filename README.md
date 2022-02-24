# tower-fault [![Latest Version]][crates.io]
[Latest Version]: https://img.shields.io/crates/v/tower-fault.svg
[crates.io]: https://crates.io/crates/tower-fault

`tower-fault` is a library for injecting various faults into a `tower::Service`.

## Layers

You can use the following layers to inject faults into a service:

* `ErrorLayer` - randomly inject errors into a service.
* `LatencyLayer` - randomly add latency into a service.

## Example usage

```rust
use tower_fault::{
    error::ErrorLayer,
    latency::LatencyLayer,
};
use tower::{service_fn, ServiceBuilder};

struct MyRequest {
    value: u64
}

async fn my_service(req: MyRequest) -> Result<(), String> {
    Ok(())
}

// LatencyLayer with a 10% probability of injecting 200 to 500 milliseconds
// of latency.
let latency_layer = LatencyLayer::new(0.1, 200..500);

// ErrorLayer that injects an error if the request value is greater than 10.
let error_layer = ErrorLayer::new(|req: &MyRequest| req.value > 10, |_: &MyRequest| String::from("error"));

let service = ServiceBuilder::new()
    .layer(latency_layer)
    .service(service_fn(my_service));
```