//! Borrowed decoded-source facts and channel envelopes, without transport or asset admission.

#[path = "envelope.rs"]
mod envelope;
#[path = "error.rs"]
mod error;
#[path = "source.rs"]
mod source;

pub use envelope::{SoundWaveEnvelope, SoundWaveEnvelopeBin};
pub use error::SoundWaveSourceError;
pub use source::{SoundWaveSource, SoundWaveSourceAdapter};

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
