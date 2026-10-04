use super::RenderMeshBounds;
use crate::core::math::{Mat4, Quat, Transform, Vec3, Vec4};

#[test]
fn render_mesh_bounds_transform_preserves_local_center_rotation_and_non_uniform_scale() {
    let local = RenderMeshBounds::from_min_max([-1.0, -2.0, -0.5], [3.0, 2.0, 0.5]);
    let transform = Transform::from_translation(Vec3::new(10.0, 20.0, 30.0))
        .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2))
        .with_scale(Vec3::new(2.0, 3.0, 4.0));

    let world = local.transformed(transform);

    assert_vec3_close(world.center, [10.0, 22.0, 30.0]);
    assert_vec3_close(world.min, [4.0, 18.0, 28.0]);
    assert_vec3_close(world.max, [16.0, 26.0, 32.0]);
    assert!((world.radius - 56.0_f32.sqrt()).abs() <= 1.0e-5);
}

#[test]
fn render_mesh_bounds_affine_matrix_projection_preserves_shear() {
    let local = RenderMeshBounds::from_min_max([-1.0, -2.0, -0.5], [3.0, 2.0, 0.5]);
    let world_from_local = Mat4::from_cols(
        Vec4::new(2.0, 0.0, 0.0, 0.0),
        Vec4::new(1.0, 1.0, 0.0, 0.0),
        Vec4::new(0.0, 0.0, 3.0, 0.0),
        Vec4::new(10.0, 20.0, 30.0, 1.0),
    );

    let world = local.transformed_by_affine(world_from_local);

    assert_vec3_close(world.center, [12.0, 20.0, 30.0]);
    assert_vec3_close(world.min, [6.0, 18.0, 28.5]);
    assert_vec3_close(world.max, [18.0, 22.0, 31.5]);
    assert!((world.radius - 6.5).abs() <= 1.0e-5);
}

#[test]
fn render_mesh_bounds_transform_rebuilds_derived_metadata_from_min_max() {
    let stale = RenderMeshBounds {
        min: [-1.0; 3],
        max: [1.0; 3],
        center: [100.0; 3],
        radius: 0.0,
    };

    let canonical = stale.transformed_by_affine(Mat4::IDENTITY);

    assert_eq!(canonical.min, [-1.0; 3]);
    assert_eq!(canonical.max, [1.0; 3]);
    assert_eq!(canonical.center, [0.0; 3]);
    assert!((canonical.radius - 3.0_f32.sqrt()).abs() <= 1.0e-5);
}

fn assert_vec3_close(actual: [f32; 3], expected: [f32; 3]) {
    for axis in 0..3 {
        assert!(
            (actual[axis] - expected[axis]).abs() <= 1.0e-5,
            "axis {axis}: expected {}, got {}",
            expected[axis],
            actual[axis]
        );
    }
}
