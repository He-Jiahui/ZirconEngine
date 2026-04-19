use crate::scene::viewport::{ProjectionMode, TransformSpace, ViewportCameraSnapshot};
use zircon_runtime::core::math::{Transform, Vec3};

use crate::scene::viewport::handles::handle_basis::HandleBasis;

pub(in crate::scene::viewport::handles) fn build_handle_basis(
    transform: Transform,
    space: TransformSpace,
    camera: &ViewportCameraSnapshot,
) -> HandleBasis {
    let extent = handle_extent(camera, transform.translation);
    let (x, y, z) = match space {
        TransformSpace::Local => (transform.right(), transform.up(), transform.forward()),
        TransformSpace::Global => (Vec3::X, Vec3::Y, -Vec3::Z),
    };
    HandleBasis {
        origin: transform,
        x,
        y,
        z,
        extent,
    }
}

pub(in crate::scene::viewport::handles) fn handle_extent(
    camera: &ViewportCameraSnapshot,
    origin: Vec3,
) -> f32 {
    match camera.projection_mode {
        ProjectionMode::Perspective => {
            (camera.transform.translation.distance(origin) * 0.22).clamp(0.75, 3.5)
        }
        ProjectionMode::Orthographic => (camera.ortho_size * 0.35).clamp(0.75, 3.5),
    }
}
