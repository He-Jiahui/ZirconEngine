//! Deterministic semantic compilation for animation authoring assets.
//!
//! The compiler is deliberately pure: loading, dependency resolution, and preview-world
//! ownership remain outside this module. Its artifact replaces string graph edges with indexes so
//! runtime evaluation and editor preview can share the same validated topology later.

mod diagnostic;
mod graph;
mod parameter;
mod product;
mod schema;
pub mod sequence;
pub mod state_machine;

pub use diagnostic::{
    AnimationCompileDiagnostic, AnimationCompileElement, AnimationCompileSeverity,
};
pub use graph::{
    compile_animation_graph, AnimationCompiledGraph, AnimationCompiledGraphNode,
    AnimationGraphCompilation,
};
pub(crate) use parameter::{parameter_kind, parameter_value_is_finite};
pub use parameter::{AnimationCompiledParameter, AnimationCompiledParameterKind};
pub use product::{compile_animation_source, AnimationCompileProduct, AnimationCompileSource};
pub use schema::{
    AnimationCompilerAssetKind, AnimationCompilerSchemaOwner, AnimationCompilerSchemaRegistry,
    AnimationCompilerSchemaVersion, AnimationGraphNodeDescriptor, AnimationGraphNodeSchemaKind,
    AnimationGraphPinDescriptor, AnimationGraphPinDirection, AnimationGraphPinValueKind,
    AnimationStateKindSchemaKind,
};

#[cfg(test)]
mod tests;
