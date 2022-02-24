use rand::Rng;
use std::{ops, time::Duration};

/// Trait that returns a random latency.
pub trait Distribution<R> {
    /// Returns a random latency.
    fn sample(&self, req: &R) -> Duration;
}

macro_rules! impl_distribution_fixed {
    ($t:ty, $ret:tt) => {
        impl<R> Distribution<R> for $t {
            fn sample(&self, _req: &R) -> Duration {
                #[allow(clippy::redundant_closure_call)]
                $ret(*self)
            }
        }
    };
}
impl_distribution_fixed! { f64, (|value| Duration::from_secs_f64(value / 1000.0)) }
impl_distribution_fixed! { u64, (Duration::from_millis) }
impl_distribution_fixed! { Duration, (|value| value) }

macro_rules! impl_distribution_range {
    ($t:ty, $ret:tt) => {
        impl<R> Distribution<R> for ops::Range<$t> {
            fn sample(&self, _req: &R) -> Duration {
                let mut rng = rand::thread_rng();
                let value = rng.gen_range(self.clone());
                #[allow(clippy::redundant_closure_call)]
                $ret(value)
            }
        }

        impl<R> Distribution<R> for ops::RangeInclusive<$t> {
            fn sample(&self, _req: &R) -> Duration {
                let mut rng = rand::thread_rng();
                let value = rng.gen_range(self.clone());
                #[allow(clippy::redundant_closure_call)]
                $ret(value)
            }
        }
    };
}
impl_distribution_range! { f64, (|value| Duration::from_secs_f64(value / 1000.0)) }
impl_distribution_range! { u64, (Duration::from_millis) }
impl_distribution_range! { Duration, (|value| value) }

impl<F, R> Distribution<R> for F
where
    F: Fn(&R) -> Duration,
{
    fn sample(&self, req: &R) -> Duration {
        self(req)
    }
}
