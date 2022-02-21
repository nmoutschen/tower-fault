use rand::prelude::*;
use std::{
    future::Future,
    marker::PhantomData,
    ops::Range,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};
use tokio::time;
use tower::{Layer, Service};

/// A layer that adds latency to the service before sending a request.
///
/// This adds a random amount of latency to a random percentage of requests.
#[derive(Debug, Clone)]
pub struct LatencyLayer<'a> {
    probability: f64,
    range: Range<u64>,
    _phantom: PhantomData<&'a ()>,
}

impl<'a> LatencyLayer<'a> {
    /// Create a new `LatencyLayer` with the given probability and latency range.
    ///
    /// The probability is the chance that a request will be delayed, bound between 0 and 1.
    /// The range is the range of latency to add, in milliseconds.
    pub fn new(probability: f64, range: Range<u64>) -> Self {
        LatencyLayer {
            probability,
            range,
            _phantom: PhantomData,
        }
    }
}

impl<'a> Default for LatencyLayer<'a> {
    fn default() -> Self {
        LatencyLayer::new(0.1, 100..200)
    }
}

impl<'a, S> Layer<S> for LatencyLayer<'a> {
    type Service = LatencyService<'a, S>;

    fn layer(&self, inner: S) -> Self::Service {
        LatencyService {
            inner,
            layer: self.clone(),
            rng: StdRng::from_entropy(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LatencyService<'a, S> {
    inner: S,
    layer: LatencyLayer<'a>,
    rng: StdRng,
}

impl<'a, R, S> Service<R> for LatencyService<'a, S>
where
    R: Send,
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
        let latency = if self.rng.gen::<f64>() < self.layer.probability {
            self.rng.gen_range(self.layer.range.clone())
        } else {
            0
        };

        let fut = self.inner.call(request);
        let fut = async move {
            time::sleep(Duration::from_millis(latency)).await;
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
