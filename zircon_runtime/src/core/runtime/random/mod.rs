//! Deterministic seed authority and random-stream execution.

mod authority;
mod derivation;
mod error;
mod lease;
mod limits;
mod registry;
mod service;
mod stream;
mod stream_error;

pub use error::RandomServiceError;
pub use lease::RandomStreamLease;
pub use limits::RandomServiceLimits;
pub use service::RandomService;
pub use stream::RandomStream;
pub use stream_error::RandomStreamError;

#[cfg(test)]
mod tests;
