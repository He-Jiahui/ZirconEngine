use zircon_runtime::core::framework::navigation::NavMeshUseGeometry;

use super::geometry::BakeGeometry;

pub(super) fn should_fallback_to_render_mesh(
    requested: NavMeshUseGeometry,
    geometry: &BakeGeometry,
) -> bool {
    matches!(requested, NavMeshUseGeometry::PhysicsColliders)
        && geometry.source_triangles() == 0
        && geometry.removed_by_modifier == 0
        && geometry.carved_by_obstacle == 0
}
