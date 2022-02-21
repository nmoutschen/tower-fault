//! Utilities for testing this crate

use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use tower::Service;

pub struct DummyService;

impl Service<()> for DummyService {
    type Response = String;
    type Error = String;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _request: ()) -> Self::Future {
        Box::pin(async { Ok(String::from("ok")) })
    }
}
