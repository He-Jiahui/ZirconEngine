use crate::core::framework::render::RenderMeshBounds;

use super::project_local_bounds_for_gpu_scene;

#[test]
fn gpu_scene_projection_preserves_off_center_mesh_local_bounds() {
    let local_bounds = RenderMeshBounds::from_min_max([2.0, -1.0, -3.0], [6.0, 3.0, 1.0]);

    let projected = project_local_bounds_for_gpu_scene(local_bounds, true);

    assert_eq!(projected.center, [4.0, 1.0, -1.0]);
    assert!((projected.radius - 12.0_f32.sqrt()).abs() <= 1.0e-6);
    assert!(!projected.force_hzb_visible);
}

#[test]
fn gpu_scene_projection_forces_temporally_varying_bounds_visible() {
    let local_bounds = RenderMeshBounds::from_min_max([-1.0; 3], [1.0; 3]);

    let projected = project_local_bounds_for_gpu_scene(local_bounds, false);

    assert_eq!(projected.center, [0.0; 3]);
    assert!((projected.radius - 3.0_f32.sqrt()).abs() <= 1.0e-6);
    assert!(projected.force_hzb_visible);
}

#[test]
fn gpu_scene_projection_sanitizes_invalid_bounds_and_fails_open() {
    let local_bounds = RenderMeshBounds {
        center: [f32::NAN, 2.0, 3.0],
        radius: f32::INFINITY,
        ..RenderMeshBounds::default()
    };

    let projected = project_local_bounds_for_gpu_scene(local_bounds, true);

    assert_eq!(projected.center, [0.0; 3]);
    assert_eq!(projected.radius, 0.0);
    assert!(projected.force_hzb_visible);
}
