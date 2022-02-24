use super::{range::Range, Error};
use rand::{distributions::Bernoulli as BernoulliDistribution, prelude::*};
use std::time::Duration;

/// Trait for deciding whether a request should be delayed and by how much.
///
/// Implementations of this trait should check if they should generate a
/// latency.
/// 
/// ## Implementation
/// 
/// The `get_latency` function should return `None` if the layer should not
/// inject any latency for this request. Otherwise, it returns a `Duration`
/// representing the delay.
pub trait Generator {
    /// Generate a random latency.
    fn get_latency(&mut self) -> Option<Duration>;
}

/// Builder for [`Generator`]s.
/// 
/// __Remark__: the `init` function could be called multiple times. Each time,
/// it should return a new [`Generator`] instance.
pub trait Builder<R> {
    /// [`Generator`] type returned by the builder
    type Generator: Generator;

    /// Initialize a new [`Generator`] with the given `rng`.
    fn init(&self, rng: R) -> Self::Generator;
}

/// [`Generator`] using a Bernoulli distribution to decide if a request should be
/// delayed.
/// 
/// The probability is the chance that a request will be delayed, bound
/// between 0 and 1. The Bernoulli disitribution is used to decide if a request
/// should be delayed.
/// 
/// The range is the range of possible delays. The delay is generated by an
/// implementation of the [`Range`] trait.
#[derive(Clone, Debug)]
pub struct Bernoulli<R, LR> {
    rng: R,
    distribution: BernoulliDistribution,
    range: LR,
}

impl<LR> Bernoulli<(), LR> {
    /// Create a new `Bernoulli` with the given probability and latency
    /// range.
    pub fn new(probability: f64, range: LR) -> Result<Self, Error> {
        Ok(Bernoulli {
            rng: (),
            distribution: BernoulliDistribution::new(probability)?,
            range,
        })
    }
}

impl<R, LR> Builder<R> for Bernoulli<(), LR>
where
    R: Rng,
    LR: Range + Clone,
{
    type Generator = Bernoulli<R, LR>;

    fn init(&self, rng: R) -> Self::Generator {
        Bernoulli {
            rng,
            distribution: self.distribution.clone(),
            range: self.range.clone(),
        }
    }
}

impl<R, LR> Generator for Bernoulli<R, LR>
where
    R: Rng,
    LR: Range,
{
    fn get_latency(&mut self) -> Option<Duration> {
        if self.distribution.sample(&mut self.rng) {
            Some(self.range.get_latency(&mut self.rng))
        } else {
            None
        }
    }
}
