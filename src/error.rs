//! # Error injection for `tower`
//!
//! Layer that injects errors randomly into a service. When an error is injected,
//! the underlying service is not called.
//!
//! ## Usage
//!
//! ```rust
//! use tower_fault_injector::error::ErrorLayer;
//! use tower::{service_fn, ServiceBuilder};
//! # struct MyRequest { value: u64 };
//! # async fn my_service(_req: MyRequest) -> Result<(), String> {
//! #     Ok(())
//! # }
//!
//! // Initialize an ErrorLayer with a 10% probability of returning
//! // an error.
//! let error_layer = ErrorLayer::new(0.1, |_: &MyRequest| String::from("error"));
//!
//! let service = ServiceBuilder::new()
//!     .layer(error_layer)
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
//! use tower_fault_injector::error::ErrorLayer;
//! # struct MyRequest { value: u64 };
//!
//! // Never inject an error.
//! ErrorLayer::new(false, |_: &MyRequest| String::from("error"));
//! // Always inject an error.
//! ErrorLayer::new(true, |_: &MyRequest| String::from("error"));
//!
//! // Inject an error 30% of the time.
//! ErrorLayer::new(0.3, |_: &MyRequest| String::from("error"));
//!
//! // Inject an error based on the request content.
//! ErrorLayer::new(|req: &MyRequest| req.value % 2 == 0, |_: &MyRequest| String::from("error"));
//! ```
//!
//! ### Generator
//!
//! The __generator__ is a function that returns an error based on the
//! request.
//!
//! ```rust
//! use tower_fault_injector::error::ErrorLayer;
//! # struct MyRequest { value: u64 };
//!
//! // Customize the error based on the request payload
//! ErrorLayer::new(false, |req: &MyRequest| format!("value: {}", req.value));
//! ```
//!

use crate::decider::Decider;
use std::{
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};
use tower::{Layer, Service};

/// Layer that randomly trigger errors for the service.
///
/// This trigger errors based on the given probability and using
/// a function to generate errors.
#[derive(Clone, Debug)]
pub struct ErrorLayer<'a, D, G> {
    decider: D,
    generator: G,
    _phantom: PhantomData<&'a ()>,
}

impl<'a> ErrorLayer<'a, (), ()> {
    /// Create a new `ErrorLayer` builder.
    pub fn builder() -> Self {
        Self {
            decider: (),
            generator: (),
            _phantom: PhantomData,
        }
    }
}

impl<'a, D, G> ErrorLayer<'a, D, G> {
    /// Create a new `ErrorLayer` builder with the given probability
    /// and error generator.
    pub fn new(decider: D, generator: G) -> Self {
        Self {
            decider,
            generator,
            _phantom: PhantomData,
        }
    }

    /// Set the given decider to be used to determine if an error
    /// should be injected.
    pub fn with_decider<ND>(self, decider: ND) -> ErrorLayer<'a, ND, G> {
        ErrorLayer {
            decider,
            generator: self.generator,
            _phantom: PhantomData,
        }
    }

    /// Set the given error generator to generate errors.
    pub fn with_generator<NG>(self, generator: NG) -> ErrorLayer<'a, D, NG> {
        ErrorLayer {
            decider: self.decider,
            generator,
            _phantom: PhantomData,
        }
    }
}

impl<'a, D, G, S> Layer<S> for ErrorLayer<'a, D, G>
where
    D: Clone,
    G: Clone,
{
    type Service = ErrorService<'a, D, G, S>;

    fn layer(&self, inner: S) -> Self::Service {
        ErrorService {
            inner,
            decider: self.decider.clone(),
            generator: self.generator.clone(),
            _phantom: PhantomData,
        }
    }
}

/// Service that randomly trigger errors instead of calling the underlying
/// service.
#[derive(Clone, Debug)]
pub struct ErrorService<'a, D, G, S> {
    inner: S,
    decider: D,
    generator: G,
    _phantom: PhantomData<&'a ()>,
}

impl<'a, D, G, S, R> Service<R> for ErrorService<'a, D, G, S>
where
    D: Decider<R> + Clone,
    G: Fn(&R) -> S::Error + Clone,
    S: Service<R> + Send,
    S::Future: Send + 'a,
    S::Error: Send + 'a,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = ErrorFuture<'a, R, S>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: R) -> Self::Future {
        if self.decider.decide(&request) {
            let error = (self.generator)(&request);
            return Box::pin(async move { Err(error) });
        }

        Box::pin(self.inner.call(request))
    }
}

type ErrorFuture<'a, R, S> = Pin<
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

    #[tokio::test]
    async fn error_success() {
        let layer = ErrorLayer::new(0.0, |_: &()| String::from("error"));
        let mut service = layer.layer(DummyService);

        for _ in 0..1000 {
            let res = service.call(()).await;
            assert_eq!(res.unwrap(), String::from("ok"));
        }
    }

    #[tokio::test]
    async fn error_fail() {
        let layer = ErrorLayer::new(1.0, |_: &()| String::from("error"));
        let mut service = layer.layer(DummyService);

        for _ in 0..1000 {
            let res = service.call(()).await;
            assert_eq!(res.unwrap_err(), String::from("error"));
        }
    }
}
