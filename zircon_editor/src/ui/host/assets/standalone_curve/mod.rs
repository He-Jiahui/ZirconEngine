//! Standalone curve source semantics over an existing DataAsset. Registry, document lifetime,
//! persistence admission and history remain with their existing owners.
//! See README.md beside this module for the source/view/mutation and semantic admission limits.

mod adapter;
mod codec;
mod equality;
mod error;
#[path = "key_structure/mod.rs"]
mod key_structure;
mod source;

pub use adapter::{StandaloneCurveAdapter, StandaloneCurveDelta, StandaloneCurveKeyEdit};
pub use codec::StandaloneCurveCodec;
pub use error::StandaloneCurveError;
pub use source::{
    StandaloneCurveChannel, StandaloneCurveChannelId, StandaloneCurveExtrapolation,
    StandaloneCurveInputUnit, StandaloneCurveInterpolation, StandaloneCurveKey,
    StandaloneCurveKind, StandaloneCurvePrecision, StandaloneCurveSource,
    StandaloneCurveTangentMode,
};

#[cfg(test)]
mod tests;
