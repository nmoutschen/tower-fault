//! # Latency injection for `tower`
//! 
//! Layer that injects latency randomly into a service.
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
//! let latency_layer = LatencyLayer::new(0.1, 200..500);
//! 
//! let service = ServiceBuilder::new()
//!     .layer(latency_layer)
//!     .service(service_fn(my_service));
//! ```
//! 
//! ### Decider
//! 
//! The __decider__ is used to determine if a latency should be injected
//! or not. This can be a boolean, float, Bernoulli distribution, a
//! closure, or a custom implementation of the [`Decider`] trait.
//! 
//! For more information, see the [`decider`](crate::decider) module.
//! 
//! ```rust
//! use tower_fault_injector::latency::LatencyLayer;
//! # struct MyRequest { value: u64 };
//! 
//! // Never inject latency.
//! LatencyLayer::new(false, 200..500);
//! // Always inject 200-500 ms of latency.
//! LatencyLayer::new(true, 200..500);
//! 
//! // Inject latency 30% of the time.
//! LatencyLayer::new(0.3, 200..500);
//! 
//! // Inject latency based on the request content.
//! LatencyLayer::new(|req: &MyRequest| req.value % 2 == 0, 200..500);
//! ```
//!
//! ### Distribution
//!
//! The latency __distribution__ is used to determine the duration of the
//! latency injected in the service. The distribution can be a `Range`,
//! `RangeInclusive`, static value, a closure, or a custom implementation
//! of the [`Distribution`] trait.
//! 
//! ```rust
//! use tower_fault_injector::latency::LatencyLayer;
//! # struct MyRequest { value: u64 };
//! 
//! // Latency between 200 and 500 milliseconds.
//! LatencyLayer::new(0.3, 200..500);
//! LatencyLayer::new(0.3, 200..=500);
//! 
//! // Latency between 200 and 500 milliseconds, using floats for
//! // extra precision.
//! LatencyLayer::new(0.3, 200.0..500.0);
//!
//! // Fixed latency of 300 milliseconds.
//! LatencyLayer::new(0.3, 300);
//! 
//! // Closure that returns a latency based on the request content.
//! LatencyLayer::new(0.3, |req: &MyRequest| req.value);
//! ```
//! 

use crate::decider::Decider;
use std::{
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::time;
use tower::{Layer, Service};

mod distribution;
pub use distribution::Distribution;

/// Layer that randomly adds latency to the service.
/// 
/// __Note__: This does not add latency to the underlying service, but rather ensure
/// that the service will have a minimal latency (set by the distribution) before
/// returning a response.
#[derive(Clone, Debug)]
pub struct LatencyLayer<'a, De, Di> {
    decider: De,
    distribution: Di,
    _phantom: PhantomData<&'a ()>,
}

impl<'a> LatencyLayer<'a, (), ()> {
    /// Create a new `LatencyLayer` builder.
    pub fn builder() -> Self {
        Self {
            decider: (),
            distribution: (),
            _phantom: PhantomData,
        }
    }
}

impl<'a, De, Di> LatencyLayer<'a, De, Di> {
    /// Create a new `LatencyLayer` builder with the given probability
    /// and latency distribution.
    pub fn new(decider: De, distribution: Di) -> Self {
        Self {
            decider,
            distribution,
            _phantom: PhantomData,
        }
    }

    /// Set the given decider to be used to determine if a latency
    /// should be injected.
    pub fn with_decider<NDe>(self, decider: NDe) -> LatencyLayer<'a, NDe, Di> {
        LatencyLayer {
            decider,
            distribution: self.distribution,
            _phantom: PhantomData,
        }
    }

    /// Set the given latency distribution to set the latency.
    pub fn with_distribution<NDi>(self, distribution: NDi) -> LatencyLayer<'a, De, NDi> {
        LatencyLayer {
            decider: self.decider,
            distribution,
            _phantom: PhantomData,
        }
    }
}

impl<'a, De, Di, S> Layer<S> for LatencyLayer<'a, De, Di>
where
    De: Clone,
    Di: Clone,
{
    type Service = LatencyService<'a, De, Di, S>;

    fn layer(&self, inner: S) -> Self::Service {
        LatencyService {
            inner,
            decider: self.decider.clone(),
            distribution: self.distribution.clone(),
            _phantom: PhantomData,
        }
    }
}

/// Service that randomly injects latency into a service.
#[derive(Clone, Debug)]
pub struct LatencyService<'a, De, Di, S> {
    inner: S,
    decider: De,
    distribution: Di,
    _phantom: PhantomData<&'a ()>,
}

impl<'a, De, Di, S, R> Service<R> for LatencyService<'a, De, Di, S>
where
    De: Decider<R> + Clone,
    Di: Distribution<R> + Clone,
    S: Service<R> + Send,
    S::Future: Send + 'a,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = LatencyFuture<'a, R, S>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: R) -> Self::Future {
        let latency = if self.decider.decide(&request) {
            Some(self.distribution.sample(&request))
        } else {
            None
        };

        let fut = self.inner.call(request);
        Box::pin(async move {
            if let Some(latency) = latency {
                time::sleep(latency).await;
            }
            fut.await
        })
    }
}

type LatencyFuture<'a, R, S> = Pin<
    Box<
        dyn Future<Output = Result<<S as Service<R>>::Response, <S as Service<R>>::Error>>
            + Send
            + 'a,
    >
>;