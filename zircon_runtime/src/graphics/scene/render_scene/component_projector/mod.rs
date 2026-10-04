mod error;
mod projection;
mod projector;
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
#[path = "tests/cases.rs"]
mod tests;
