use rand::Rng;
use std::{ops, time::Duration};

/// Trait for types that can generate a random latency.
///
/// Implementations of this trait should not check if they should generate a
/// latency or not, but simply return a random latency.
pub trait Range {
    /// Return a random latency within the range.
    fn get_latency<R: Rng>(&self, rng: &mut R) -> Duration;
}

impl Range for ops::Range<u64> {
    fn get_latency<R: Rng>(&self, rng: &mut R) -> Duration {
        Duration::from_millis(rng.gen_range(self.clone()))
    }
}

impl Range for ops::Range<f32> {
    fn get_latency<R: Rng>(&self, rng: &mut R) -> Duration {
        Duration::from_secs_f32(rng.gen_range(self.clone()) / 1000.0)
    }
}

impl Range for ops::Range<f64> {
    fn get_latency<R: Rng>(&self, rng: &mut R) -> Duration {
        Duration::from_secs_f64(rng.gen_range(self.clone()) / 1000.0)
    }
}

impl Range for ops::Range<Duration> {
    fn get_latency<R: Rng>(&self, rng: &mut R) -> Duration {
        rng.gen_range(self.clone())
    }
}

impl Range for ops::RangeInclusive<u64> {
    fn get_latency<R: Rng>(&self, rng: &mut R) -> Duration {
        Duration::from_millis(rng.gen_range(self.clone()))
    }
}

impl Range for ops::RangeInclusive<f32> {
    fn get_latency<R: Rng>(&self, rng: &mut R) -> Duration {
        Duration::from_secs_f32(rng.gen_range(self.clone()) / 1000.0)
    }
}

impl Range for ops::RangeInclusive<f64> {
    fn get_latency<R: Rng>(&self, rng: &mut R) -> Duration {
        Duration::from_secs_f64(rng.gen_range(self.clone()) / 1000.0)
    }
}

impl Range for ops::RangeInclusive<Duration> {
    fn get_latency<R: Rng>(&self, rng: &mut R) -> Duration {
        rng.gen_range(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::prelude::*;
    use std::time::Duration;

    macro_rules! latency_range {
        ($name:expr, $val:expr, $cmp1:expr, $cmp2:expr) => {
            paste::paste! {
                #[tokio::test]
                async fn [<latency_range_ $name>]() {
                    let mut rng = StdRng::from_entropy();

                    for _ in 0..100 {
                        let val = ($val).get_latency(&mut rng);
                        dbg!(val);
                        assert!(val.[<$cmp1>](&Duration::from_millis(10)));
                        assert!(val.[<$cmp2>](&Duration::from_millis(20)));
                    }
                }
            }
        };
    }

    latency_range!("u64", 10..20, "ge", "lt");
    latency_range!("inclusive_u64", 10..=20, "ge", "le");
    latency_range!("f32", 10.0..20.0, "ge", "lt");
    latency_range!("inclusive_f32", 10.0..=20.0, "ge", "le");
    latency_range!("f64", 10.0..20.0, "ge", "lt");
    latency_range!("inclusive_f64", 10.0..=20.0, "ge", "le");
    latency_range!(
        "duration",
        Duration::from_millis(10)..Duration::from_millis(20),
        "ge",
        "lt"
    );
    latency_range!(
        "inclusive_duration",
        Duration::from_millis(10)..=Duration::from_millis(20),
        "ge",
        "le"
    );
}
