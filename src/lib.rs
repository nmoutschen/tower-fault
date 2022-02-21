#[cfg(feature = "latency")]
mod latency;
#[cfg(feature = "latency")]
pub use latency::{LatencyLayer, LatencyService};
