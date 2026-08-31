mod error;
mod projection;
mod projector;
mod resolver;

pub(crate) use error::{RenderSceneComponentProjectionError, RenderSceneRequiredComponent};
pub(crate) use projector::RenderSceneComponentProjector;
pub(crate) use resolver::{
    RenderSceneGeometryResolveIssue, RenderSceneGeometryResolver, RenderSceneResolvedGeometry,
};

#[cfg(test)]
mod tests;
