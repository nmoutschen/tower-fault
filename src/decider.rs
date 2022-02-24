//! # Decider
//!
//! This module contains the [`Decider`] trait, which decides if a fault should
//! be injected for a given request or response. It adds a `decide` method that
//! returns a boolean based on the request.
//!
//! For primitive that implements this trait, such as `f64` and `bool`, it
//! doesn't look at the request.
//!
//! ## Example
//!
//! ```rust
//! use tower_fault::decider::Decider;
//! # struct MyRequest { value: u64 };
//! # impl MyRequest {
//! #     fn new(value: u64) -> Self {
//! #         Self { value }
//! #     }
//! # }
//!
//! let my_request = MyRequest::new(6);
//!
//! // Always.
//! assert_eq!(true, true.decide(&my_request));
//!
//! // Never.
//! assert_eq!(false, false.decide(&my_request));
//!
//! // 30% of the time.
//! let decision = (0.3).decide(&my_request);
//!
//! // Based on the request, using a closure as decider.
//! let decision = (|req: &MyRequest| req.value % 2 == 0).decide(&my_request);
//! ```

use rand::{
    distributions::{Bernoulli, Distribution},
    Rng,
};

/// Trait for deciding if a fault should be injected for a given request or
/// response.
pub trait Decider<R> {
    /// Decide if a fault should be injected for a given request or response.
    fn decide(&self, req: &R) -> bool;
}

impl<R> Decider<R> for bool {
    fn decide(&self, _: &R) -> bool {
        *self
    }
}

impl<R> Decider<R> for Bernoulli {
    fn decide(&self, _: &R) -> bool {
        self.sample(&mut rand::thread_rng())
    }
}

impl<R> Decider<R> for f64 {
    fn decide(&self, _: &R) -> bool {
        (&mut rand::thread_rng()).gen_bool(*self)
    }
}

impl<F, R> Decider<R> for F
where
    F: Fn(&R) -> bool,
{
    fn decide(&self, req: &R) -> bool {
        self(req)
    }
}
