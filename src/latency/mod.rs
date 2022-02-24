//! # Latency injection for `tower`
//!
//! Layer that injects a random amount of latency into a service.
//!
//! ## Usage
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
//! let latency_layer = LatencyLayer::new_with_bernoulli(0.1, 200..500).unwrap();
//!
//! let service = ServiceBuilder::new()
//!     .layer(latency_layer)
//!     .service(service_fn(my_service));
//! ```

use rand::prelude::*;
use std::{
    future::Future,
    marker::PhantomData,
    ops,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::time;
use tower::{Layer, Service};

mod error;
pub use error::*;

mod generator;
pub use generator::{Builder as GeneratorBuilder, Generator, Bernoulli};

mod range;
pub use range::Range;

/// A layer that adds latency to the service before sending a request.
///
/// This adds a random amount of latency to a random percentage of requests.
#[derive(Debug, Clone)]
pub struct LatencyLayer<'a, G> {
    generator: G,
    _phantom: PhantomData<&'a ()>,
}

impl<'a, G> LatencyLayer<'a, G> {
    /// Create a new `LatencyLayer` with the given [`GeneratorBuilder`].
    pub fn new(generator: G) -> Self {
        LatencyLayer {
            generator,
            _phantom: PhantomData,
        }
    }
}

impl<'a, R> LatencyLayer<'a, generator::Bernoulli<(), R>> {
    /// Create a new `LatencyLayer` with the given probability and latency range.
    ///
    /// The probability is the chance that a request will be delayed, bound
    /// between 0 and 1. A probability of 0.5 means that 50% of the calls
    /// to the service will result in elevated latencies.
    ///
    /// The range is the range of latency to add, in milliseconds.
    pub fn new_with_bernoulli(probability: f64, range: R) -> Result<Self, Error> {
        Ok(LatencyLayer::new(generator::Bernoulli::new(
            probability,
            range,
        )?))
    }
}

impl<'a> Default for LatencyLayer<'a, generator::Bernoulli<(), ops::Range<u64>>> {
    fn default() -> Self {
        LatencyLayer::new_with_bernoulli(0.1, 100..200)
            .expect("failed to create default latency layer")
    }
}

impl<'a, S, G> Layer<S> for LatencyLayer<'a, G>
where
    G: generator::Builder<StdRng>,
{
    type Service = LatencyService<'a, S, G::Generator>;

    fn layer(&self, inner: S) -> Self::Service {
        LatencyService {
            inner,
            generator: self.generator.init(StdRng::from_entropy()),
            _phantom: PhantomData,
        }
    }
}

/// Underlying service for the `LatencyLayer`
#[derive(Debug, Clone)]
pub struct LatencyService<'a, S, G> {
    inner: S,
    generator: G,
    _phantom: PhantomData<&'a ()>,
}

impl<'a, R, S, G> Service<R> for LatencyService<'a, S, G>
where
    R: Send,
    G: generator::Generator,
    S: Service<R> + Send,
    S::Future: Send + 'a,
    S::Response: Send,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = LatencyFuture<'a, R, S>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: R) -> Self::Future {
        // Calculate latency
        let latency = self.generator.get_latency();

        let fut = self.inner.call(request);
        let fut = async move {
            if let Some(latency) = latency {
                time::sleep(latency).await;
            }
            fut.await
        };

        Box::pin(fut)
    }
}

type LatencyFuture<'a, R, S> = Pin<
    Box<
        dyn Future<Output = Result<<S as Service<R>>::Response, <S as Service<R>>::Error>>
            + Send
            + 'a,
    >,
>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;
    use std::time::{Duration, Instant};

    #[tokio::test]
    async fn latency_none() -> Result<(), Error> {
        let latency = LatencyLayer::new_with_bernoulli(0.0, 10..20)?;
        let mut service = latency.layer(DummyService);

        for _ in 0..1000 {
            let now = Instant::now();
            let _res = service.call(()).await;
            let elapsed = now.elapsed();

            assert!(elapsed < Duration::from_millis(5));
        }

        Ok(())
    }

    #[tokio::test]
    async fn latency_all() -> Result<(), Error> {
        let latency = LatencyLayer::new_with_bernoulli(1.0, 10..11)?;
        let mut service = latency.layer(DummyService);

        for _ in 0..100 {
            let now = Instant::now();
            let _res = service.call(()).await;
            let elapsed = now.elapsed();

            assert!(elapsed > Duration::from_millis(5));
        }

        Ok(())
    }
}
