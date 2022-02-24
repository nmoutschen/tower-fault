#![warn(missing_debug_implementations, missing_docs, unreachable_pub)]
#![cfg_attr(docsrs, feature(doc_cfg))]
//! # Fault injection utilities for `tower`
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
//! use tower_fault::{
//! error::ErrorLayer,
//! latency::LatencyLayer,
//! };
//! use tower::{service_fn, ServiceBuilder};
//!
//! # struct MyRequest {
//! #     value: u64,
//! # }
//!
//! # async fn my_service(req: MyRequest) -> Result<(), String> {
//! #     Ok(())
//! # }
//!
//! // LatencyLayer with a 10% probability of injecting 200 to 500 milliseconds
//! // of latency.
//! let latency_layer = LatencyLayer::new(0.1, 200..500);
//!
//! // ErrorLayer that injects an error if the request value is greater than 10.
//! let error_layer = ErrorLayer::new(
//!     |req: &MyRequest| req.value > 10,
//!     |_: &MyRequest| String::from("error")
//! );
//!
//! let service = ServiceBuilder::new()
//!     .layer(latency_layer)
//!     .layer(error_layer)
//!     .service(service_fn(my_service));
//! ```

#[cfg(feature = "error")]
#[cfg_attr(docsrs, doc(cfg(feature = "error")))]
pub mod error;

#[cfg(feature = "latency")]
#[cfg_attr(docsrs, doc(cfg(feature = "latency")))]
pub mod latency;

pub mod decider;

#[cfg(test)]
mod test_utils;
