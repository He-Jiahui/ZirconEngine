#[path = "error.rs"]
mod error;
#[path = "projection.rs"]
mod projection;
#[path = "projector.rs"]
mod projector;
#[path = "resolver.rs"]
mod resolver;

pub(crate) use error::{
    RenderSceneComponentProjectionError, RenderSceneComponentProjectionTransactionError,
    RenderSceneRequiredComponent,
};
pub(crate) use projector::{RenderSceneComponentProjectionCommit, RenderSceneComponentProjector};
pub(crate) use resolver::{
    RenderSceneGeometryResolveIssue, RenderSceneGeometryResolver, RenderSceneResolvedGeometry,
};

#[cfg(test)]
#[path = "../../tests/component_projector_cases.rs"]
mod tests;
