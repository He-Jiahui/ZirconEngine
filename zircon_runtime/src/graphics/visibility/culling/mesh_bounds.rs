use super::super::declarations::VisibilityBounds;
use crate::core::framework::render::{RenderMeshBounds, RenderMeshSnapshot};
use crate::core::math::Vec3;

const MIN_GPU_BOUNDS_RADIUS_SCALE: f32 = 0.0001;

/// Projects the prepared mesh bounds into world space. Missing or invalid
/// geometry bounds deliberately fail open so CPU visibility cannot discard a
/// primitive while its resource generation is being admitted.
pub(crate) fn mesh_bounds_from_local(
    mesh: &RenderMeshSnapshot,
    local_bounds: Option<RenderMeshBounds>,
) -> VisibilityBounds {
    let Some(local_bounds) = local_bounds.filter(|bounds| {
        bounds.center.iter().all(|value| value.is_finite())
            && bounds.radius.is_finite()
            && bounds.radius >= 0.0
    }) else {
        return fail_open_bounds(mesh);
    };

    let world_from_local = mesh.transform.matrix();
    let center = world_from_local.transform_point3(Vec3::from_array(local_bounds.center));
    let radius_scale = world_from_local
        .x_axis
        .truncate()
        .length()
        .max(world_from_local.y_axis.truncate().length())
        .max(world_from_local.z_axis.truncate().length())
        .max(MIN_GPU_BOUNDS_RADIUS_SCALE);
    let radius = local_bounds.radius * radius_scale;
    if center.is_finite() && radius.is_finite() && radius >= 0.0 {
        VisibilityBounds { center, radius }
    } else {
        fail_open_bounds(mesh)
    }
}

fn fail_open_bounds(mesh: &RenderMeshSnapshot) -> VisibilityBounds {
    VisibilityBounds {
        center: if mesh.transform.translation.is_finite() {
            mesh.transform.translation
        } else {
            Vec3::ZERO
        },
        radius: f32::MAX,
    }
}

#[cfg(test)]
#[path = "tests/mesh_bounds.rs"]
mod tests;
