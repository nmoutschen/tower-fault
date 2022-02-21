use rand::prelude::*;
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
    probability: f64,
    func: F,
    _phantom: PhantomData<&'a ()>,
}

impl<'a, F> ErrorLayer<'a, F> {
    /// Create a new `ErrorLayer` with the given probability and error function.
    ///
    /// The probability is the chance that a request will result in an error,
    /// bound between 0 and 1.
    pub fn new(probability: f64, func: F) -> Self {
        ErrorLayer {
            probability,
            func,
            _phantom: PhantomData,
        }
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
        if self.rng.gen::<f64>() < self.layer.probability {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;

    #[tokio::test]
    async fn error_success() {
        let layer = ErrorLayer::new(0.0, || String::from("error"));
        let mut service = layer.layer(DummyService);

        for _ in 0..1000 {
            let res = service.call(()).await;
            assert_eq!(res.unwrap(), String::from("ok"));
        }
    }

    #[tokio::test]
    async fn error_fail() {
        let layer = ErrorLayer::new(1.0, || String::from("error"));
        let mut service = layer.layer(DummyService);

        for _ in 0..1000 {
            let res = service.call(()).await;
            assert_eq!(res.unwrap_err(), String::from("error"));
        }
    }
}
