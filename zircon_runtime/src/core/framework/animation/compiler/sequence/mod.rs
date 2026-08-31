//! Sequence semantic validation and canonical track IR.

mod compile;
mod model;

pub use compile::compile_animation_sequence;
pub use model::{
    AnimationCompiledSequence, AnimationCompiledSequenceBinding, AnimationCompiledSequenceKey,
    AnimationCompiledSequenceTrack, AnimationCompiledSequenceValueKind,
    AnimationSequenceCompilation,
};

#[cfg(test)]
mod tests;
