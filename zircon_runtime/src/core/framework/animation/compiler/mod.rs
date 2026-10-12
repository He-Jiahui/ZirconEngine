//! Deterministic semantic compilation for animation authoring assets.
//!
//! The compiler is deliberately pure: loading, dependency resolution, and preview-world
//! ownership remain outside this module. Its artifact replaces string graph edges with indexes so
//! runtime evaluation and editor preview can share the same validated topology later.

#[path = "diagnostic.rs"]
mod diagnostic;
#[path = "graph.rs"]
mod graph;
#[path = "parameter.rs"]
mod parameter;
#[path = "product.rs"]
mod product;
#[path = "schema.rs"]
mod schema;
#[path = "sequence/mod.rs"]
pub mod sequence;
#[path = "state_machine/mod.rs"]
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
#[path = "tests/cases.rs"]
mod tests;
