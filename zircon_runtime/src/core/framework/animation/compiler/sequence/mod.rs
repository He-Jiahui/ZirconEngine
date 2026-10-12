//! Sequence semantic validation and canonical track IR.

#[path = "compile.rs"]
mod compile;
#[path = "model.rs"]
mod model;

pub use compile::compile_animation_sequence;
pub use model::{
    AnimationCompiledSequence, AnimationCompiledSequenceBinding, AnimationCompiledSequenceKey,
    AnimationCompiledSequenceTrack, AnimationCompiledSequenceValueKind,
    AnimationSequenceCompilation,
};

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
