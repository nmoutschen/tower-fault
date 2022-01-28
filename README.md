# tower-fault-injector [![Latest Version]][crates.io]
[Latest Version]: https://img.shields.io/crates/v/tower-fault-injector.svg
[crates.io]: https://crates.io/crates/tower-fault-injector

`tower-fault-injector` is a library for injecting various faults into a tower Service.

```rust,no-run
use tower::ServiceBuilder;
use tower_fault_injector::LatencyLayer;

fn main() {
    // Create a latency layer with default parameters
    let latency_layer = LatencyLayer::default();

    // Create a service that uses the latency layer
    let my_service = ServiceBuilder::new()
        .layer(latency_layer)
        .service(MyService::new());
}
```