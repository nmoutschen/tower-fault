# tower-fault-injector [![Latest Version]][crates.io]
[Latest Version]: https://img.shields.io/crates/v/tower-fault-injector.svg
[crates.io]: https://crates.io/crates/tower-fault-injector

`tower-fault-injector` is a library for injecting various faults into a `tower::Service`.

## Layers

You can use the following layers to inject faults into a service:

* `ErrorLayer` - randomly inject errors into a service.
* `LatencyLayer` - randomly add latency into a service.

## Example usage

```rust
use tower_fault_injector::latency::LatencyLayer;
use tower::{service_fn, ServiceBuilder};

async fn my_service() -> Result<(), ()> {
    Ok(())
}

// Initialize a LatencyLayer with a 10% probability of injecting
// 200 to 500 milliseconds of latency.
let latency_layer = LatencyLayer::new(0.1, 200..500);

let service = ServiceBuilder::new()
    .layer(latency_layer)
    .service(service_fn(my_service));
```