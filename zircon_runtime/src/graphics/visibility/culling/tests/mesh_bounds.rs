use super::{mesh_bounds_from_local, MIN_GPU_BOUNDS_RADIUS_SCALE};
use crate::core::framework::render::{RenderMeshBounds, RenderMeshSnapshot};
use crate::core::framework::scene::Mobility;
use crate::core::math::{Quat, Transform, Vec3, Vec4};
use crate::core::resource::{MaterialMarker, ModelMarker, ResourceHandle, ResourceId};

#[test]
fn cpu_bounds_use_off_center_local_bounds_after_affine_transform() {
    let mesh = test_mesh(
        Transform::from_translation(Vec3::new(10.0, 20.0, 30.0))
            .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2))
            .with_scale(Vec3::new(-2.0, 3.0, -4.0)),
    );
    let bounds = mesh_bounds_from_local(
        &mesh,
        Some(RenderMeshBounds::from_min_max(
            [2.0, -1.0, -3.0],
            [6.0, 3.0, 1.0],
        )),
    );
    assert!(
        (bounds.center - Vec3::new(7.0, 12.0, 34.0))
            .abs()
            .max_element()
            <= 1.0e-5
    );
    assert!((bounds.radius - 192.0_f32.sqrt()).abs() <= 1.0e-5);
}

#[test]
fn cpu_bounds_fail_open_when_prepared_bounds_are_missing_or_invalid() {
    let mesh = test_mesh(Transform::from_translation(Vec3::new(3.0, 4.0, 5.0)));
    let missing = mesh_bounds_from_local(&mesh, None);
    assert_eq!(missing.center, Vec3::new(3.0, 4.0, 5.0));
    assert_eq!(missing.radius, f32::MAX);
    let invalid = mesh_bounds_from_local(
        &mesh,
        Some(RenderMeshBounds::from_min_max([f32::NAN; 3], [1.0; 3])),
    );
    assert_eq!(invalid.radius, f32::MAX);

    let overflowed = mesh_bounds_from_local(
        &test_mesh(Transform::default().with_scale(Vec3::splat(f32::MAX))),
        Some(RenderMeshBounds::from_min_max(
            [-f32::MAX; 3],
            [f32::MAX; 3],
        )),
    );
    assert!(overflowed.center.is_finite());
    assert_eq!(overflowed.radius, f32::MAX);

    let non_finite_transform = mesh_bounds_from_local(
        &test_mesh(Transform::from_translation(Vec3::splat(f32::NAN))),
        Some(RenderMeshBounds::from_min_max([-1.0; 3], [1.0; 3])),
    );
    assert_eq!(non_finite_transform.center, Vec3::ZERO);
    assert_eq!(non_finite_transform.radius, f32::MAX);

    let degenerate = mesh_bounds_from_local(
        &test_mesh(Transform::default().with_scale(Vec3::ZERO)),
        Some(RenderMeshBounds::from_min_max([-1.0; 3], [1.0; 3])),
    );
    assert!(
        (degenerate.radius - 3.0_f32.sqrt() * MIN_GPU_BOUNDS_RADIUS_SCALE).abs() <= f32::EPSILON
    );
}

fn test_mesh(transform: Transform) -> RenderMeshSnapshot {
    RenderMeshSnapshot {
        node_id: 1,
        stable_instance_key: 2,
        transform_revision: 1,
        transform,
        model: ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label(
            "mesh-bounds/model",
        )),
        mesh: None,
        material: ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(
            "mesh-bounds/material",
        )),
        mesh_lod: None,
        morph_weights: Vec::new(),
        tint: Vec4::ONE,
        mobility: Mobility::Static,
        static_state: Default::default(),
        common: Default::default(),
    }
}
