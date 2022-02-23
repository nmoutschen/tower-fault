//! # Error injection for `tower`
//!
//! Layer that injects errors randomly into a service.
//!
//! ## Usage
//!
//! ```rust
//! use tower_fault_injector::error::ErrorLayer;
//! use tower::{service_fn, ServiceBuilder};
//! # async fn my_service() -> Result<(), String> {
//! #     Ok(())
//! # }
//!
//! // Initialize an ErrorLayer with a 10% probability of returning
//! // an error.
//! let error_layer = ErrorLayer::new(0.1, || String::from("error")).unwrap();
//!
//! let service = ServiceBuilder::new()
//!     .layer(error_layer)
//!     .service(service_fn(my_service));
//! ```

use rand::{
    distributions::{Bernoulli, BernoulliError},
    prelude::*,
};
use std::{
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};
use tower::{Layer, Service};

/// A layer that randomly trigger errors for the service.
///
/// This trigger errors based on the given probability and using
/// a function to generate errors.
#[derive(Clone, Debug)]
pub struct ErrorLayer<'a, F> {
    distribution: Bernoulli,
    func: F,
    _phantom: PhantomData<&'a ()>,
}

impl<'a, F> ErrorLayer<'a, F> {
    /// Create a new `ErrorLayer` with the given probability and error function.
    ///
    /// The probability is the chance that a request will result in an error,
    /// bound between 0 and 1. A probability of 0.5 means that 50% of the calls
    /// to the service will result in an error.
    pub fn new(probability: f64, func: F) -> Result<Self, Error> {
        Ok(ErrorLayer {
            distribution: Bernoulli::new(probability)?,
            func,
            _phantom: PhantomData,
        })
    }
}

impl<'a, F, S> Layer<S> for ErrorLayer<'a, F>
where
    F: Clone,
{
    type Service = ErrorService<'a, F, S>;

    fn layer(&self, inner: S) -> Self::Service {
        ErrorService {
            inner,
            layer: self.clone(),
            rng: StdRng::from_entropy(),
        }
    }
}

/// Service that randomly trigger errors instead of calling the underlying
/// service.
#[derive(Clone, Debug)]
pub struct ErrorService<'a, F, S> {
    inner: S,
    layer: ErrorLayer<'a, F>,
    rng: StdRng,
}

impl<'a, F, S, R> Service<R> for ErrorService<'a, F, S>
where
    R: Send,
    S: Service<R> + Send,
    S::Future: Send + 'a,
    S::Error: Send + 'a,
    S::Response: Send,
    F: Fn() -> S::Error,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = ErrorFuture<'a, R, S>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: R) -> Self::Future {
        if self.layer.distribution.sample(&mut self.rng) {
            let error = (self.layer.func)();
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

/// Errors that can be returned by the `ErrorLayer`.
#[derive(Debug)]
pub enum Error {
    /// Error creating an `ErrorLayer`
    NewLayerError(&'static str),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NewLayerError(s) => write!(f, "cannot create the layer: {}", s),
        }
    }
}

impl From<BernoulliError> for Error {
    fn from(_err: BernoulliError) -> Self {
        Error::NewLayerError("invalid probability")
    }
}

impl std::error::Error for Error {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;

    #[tokio::test]
    async fn error_success() -> Result<(), Error> {
        let layer = ErrorLayer::new(0.0, || String::from("error"))?;
        let mut service = layer.layer(DummyService);

        for _ in 0..1000 {
            let res = service.call(()).await;
            assert_eq!(res.unwrap(), String::from("ok"));
        }

        Ok(())
    }

    #[tokio::test]
    async fn error_fail() -> Result<(), Error> {
        let layer = ErrorLayer::new(1.0, || String::from("error"))?;
        let mut service = layer.layer(DummyService);

        for _ in 0..1000 {
            let res = service.call(()).await;
            assert_eq!(res.unwrap_err(), String::from("error"));
        }

        Ok(())
    }
}
