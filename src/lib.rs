//! Fault injection utilities for `tower`
//!
//! This crate provides [`tower::Layer`]s that can be used to inject various
//! faults into a [`tower::Service`].
//!
//! ## Layers
//!
//! You can use the following layers to inject faults into a service:
//!
//! * [`ErrorLayer`](error/struct.ErrorLayer.html) - randomly inject errors into a service.
//! * [`LatencyLayer`](latency/struct.LatencyLayer.html) - randomly add latency into a service.
//!
//! ## Example
//!
//! ```rust
//! use tower_fault_injector::latency::LatencyLayer;
//! use tower::{service_fn, ServiceBuilder};
//! # async fn my_service() -> Result<(), ()> {
//! #     Ok(())
//! # }
//!
//! // Initialize a LatencyLayer with a 10% probability of injecting
//! // 200 to 500 milliseconds of latency.
//! let latency_layer = LatencyLayer::new(0.1, 200..500);
//!
//! let service = ServiceBuilder::new()
//!     .layer(latency_layer)
//!     .service(service_fn(my_service));
//! ```

#[cfg(feature = "latency")]
#[cfg_attr(docsrs, doc(cfg(feature = "latency")))]
pub mod latency;

#[cfg(feature = "error")]
#[cfg_attr(docsrs, doc(cfg(feature = "error")))]
pub mod error;

#[cfg(test)]
mod test_utils;
