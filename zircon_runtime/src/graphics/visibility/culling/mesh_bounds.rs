use super::super::declarations::VisibilityBounds;
use crate::core::framework::render::RenderMeshSnapshot;

pub(crate) fn mesh_bounds(mesh: &RenderMeshSnapshot) -> VisibilityBounds {
    VisibilityBounds {
        center: mesh.transform.translation,
        radius: mesh.transform.scale.abs().length() * 0.5,
    }
}
