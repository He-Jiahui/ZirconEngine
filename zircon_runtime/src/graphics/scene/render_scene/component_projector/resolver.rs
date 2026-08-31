use std::fmt;

use crate::core::framework::scene::EntityId;

use super::super::{
    RenderSceneMeshSource, RenderScenePrimitiveLocalBounds, RenderScenePrimitiveRevisions,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RenderSceneGeometryResolveIssue {
    Pending,
    Missing,
    Invalid,
}

impl fmt::Display for RenderSceneGeometryResolveIssue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Pending => "pending",
            Self::Missing => "missing",
            Self::Invalid => "invalid",
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RenderSceneResolvedGeometry {
    local_bounds: RenderScenePrimitiveLocalBounds,
    geometry_revision: u64,
    bounds_revision: u64,
    deformation_revision: u64,
}

impl RenderSceneResolvedGeometry {
    pub(crate) const fn new(
        local_bounds: RenderScenePrimitiveLocalBounds,
        geometry_revision: u64,
        bounds_revision: u64,
        deformation_revision: u64,
    ) -> Self {
        Self {
            local_bounds,
            geometry_revision,
            bounds_revision,
            deformation_revision,
        }
    }

    pub(super) fn apply_to(
        self,
        revisions: &mut RenderScenePrimitiveRevisions,
    ) -> RenderScenePrimitiveLocalBounds {
        revisions.geometry = self.geometry_revision;
        revisions.bounds = self.bounds_revision;
        revisions.deformation = self.deformation_revision;
        self.local_bounds
    }
}

pub(crate) trait RenderSceneGeometryResolver {
    fn resolve_geometry(
        &mut self,
        entity: EntityId,
        source: &RenderSceneMeshSource,
        morph_weights: &[f32],
    ) -> Result<RenderSceneResolvedGeometry, RenderSceneGeometryResolveIssue>;
}
