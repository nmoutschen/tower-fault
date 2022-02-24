use rand::distributions::BernoulliError;

/// Errors that can be returned by the `LatencyLayer`.
#[derive(Debug)]
pub enum Error {
    /// Error creating an `LatencyLayer`
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
