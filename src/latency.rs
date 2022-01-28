use rand::prelude::*;
use std::{
    ops::Range,
    task::{Context, Poll},
    thread::sleep,
    time::Duration,
};
use tower::{Layer, Service};

/// A layer that adds latency to the service before sending a request.
///
/// This adds a random amount of latency to a random percentage of requests.
#[derive(Debug, Clone)]
pub struct LatencyLayer {
    probability: f64,
    range: Range<u64>,
}

impl LatencyLayer {
    /// Create a new `LatencyLayer` with the given probability and latency range.
    ///
    /// The probability is the chance that a request will be delayed, bound between 0 and 1.
    /// The range is the range of latency to add, in milliseconds.
    pub fn new(probability: f64, range: Range<u64>) -> Self {
        LatencyLayer { probability, range }
    }
}

impl Default for LatencyLayer {
    fn default() -> Self {
        LatencyLayer::new(0.1, 100..200)
    }
}

impl<S> Layer<S> for LatencyLayer {
    type Service = LatencyService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        LatencyService {
            inner,
            latency: self.clone(),
            rng: thread_rng(),
        }
    }
}

pub struct LatencyService<S> {
    inner: S,
    latency: LatencyLayer,
    rng: ThreadRng,
}

impl<S, R> Service<R> for LatencyService<S>
where
    S: Service<R>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: R) -> Self::Future {
        if self.rng.gen::<f64>() < self.latency.probability {
            let latency = self.rng.gen_range(self.latency.range.clone());
            sleep(Duration::from_millis(latency));
        }
        self.inner.call(request)
    }
}
