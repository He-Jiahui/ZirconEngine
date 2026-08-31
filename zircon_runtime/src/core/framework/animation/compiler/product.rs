//! Unified source/product dispatch for shared animation compilation.

use crate::core::framework::animation::{
    AnimationGraphAsset, AnimationSequenceAsset, AnimationStateMachineAsset,
};

use super::sequence::{compile_animation_sequence, AnimationSequenceCompilation};
use super::state_machine::{compile_animation_state_machine, AnimationStateMachineCompilation};
use super::{compile_animation_graph, AnimationCompileDiagnostic, AnimationGraphCompilation};

/// Borrowed source accepted by the shared animation compiler boundary.
#[derive(Clone, Copy, Debug)]
pub enum AnimationCompileSource<'a> {
    Sequence(&'a AnimationSequenceAsset),
    Graph(&'a AnimationGraphAsset),
    StateMachine(&'a AnimationStateMachineAsset),
}

/// Typed result produced by one shared animation compiler invocation.
#[derive(Clone, Debug, PartialEq)]
pub enum AnimationCompileProduct {
    Sequence(AnimationSequenceCompilation),
    Graph(AnimationGraphCompilation),
    StateMachine(AnimationStateMachineCompilation),
}

impl AnimationCompileProduct {
    /// Whether this source revision produced an installable source-only compiler artifact.
    pub fn is_successful(&self) -> bool {
        match self {
            Self::Sequence(compilation) => compilation.artifact().is_some(),
            Self::Graph(compilation) => compilation.artifact().is_some(),
            Self::StateMachine(compilation) => compilation.artifact().is_some(),
        }
    }

    pub fn diagnostics(&self) -> &[AnimationCompileDiagnostic] {
        match self {
            Self::Sequence(compilation) => compilation.diagnostics(),
            Self::Graph(compilation) => compilation.diagnostics(),
            Self::StateMachine(compilation) => compilation.diagnostics(),
        }
    }
}

/// Dispatches a source asset through the compiler-owned semantic contract.
///
/// Loading bytes, resolving external resources, generation control, and artifact installation are
/// deliberate callers of this boundary rather than compiler responsibilities.
pub fn compile_animation_source(source: AnimationCompileSource<'_>) -> AnimationCompileProduct {
    match source {
        AnimationCompileSource::Sequence(sequence) => {
            AnimationCompileProduct::Sequence(compile_animation_sequence(sequence))
        }
        AnimationCompileSource::Graph(graph) => {
            AnimationCompileProduct::Graph(compile_animation_graph(graph))
        }
        AnimationCompileSource::StateMachine(state_machine) => {
            AnimationCompileProduct::StateMachine(compile_animation_state_machine(state_machine))
        }
    }
}
