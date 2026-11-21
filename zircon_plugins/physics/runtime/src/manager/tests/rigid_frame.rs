use crate::build_world_sync_state;
use std::f32::consts::FRAC_PI_4;
use zircon_runtime::core::framework::scene::WorldHandle;
use zircon_runtime::core::math::{Quat, Real, Transform, Vec3};
use zircon_runtime::scene::components::{
    ColliderComponent, ColliderShape, NodeKind, RigidBodyComponent,
};
use zircon_runtime::scene::world::World;

const POINT_EPS: Real = 1e-4;
const AXIS_EPS: Real = 1e-4;
const UNIT_EPS: Real = 1e-5;

#[test]
fn positive_shear_exports_the_polar_body_basis() {
    planar("positive", 2.0, FRAC_PI_4, Vec3::X * 8.0);
}

#[test]
fn mirrored_shear_exports_the_signed_x_polar_basis() {
    planar("mirrored", -2.0, -FRAC_PI_4, Vec3::X * 8.0);
}

#[test]
fn mixed_axis_shear_exports_the_matrix_basis_not_the_authored_rotation_chain() {
    // In XY, D1 * Rz(45) * D2 = [[12,-4],[3,1]] / sqrt(2).
    // Its proper polar angle is atan2(3+4,12+1); the final V is proper orthogonal.
    let outer = Transform {
        translation: Vec3::new(1.0, 2.0, 3.0),
        scale: Vec3::new(4.0, 1.0, 2.0),
        ..Transform::identity()
    };
    let inner = Transform {
        rotation: Quat::from_rotation_z(FRAC_PI_4),
        scale: Vec3::new(3.0, 1.0, 1.0),
        ..Transform::identity()
    };
    let v = Quat::from_rotation_x(FRAC_PI_4) * Quat::from_rotation_y(FRAC_PI_4);
    let child = Transform {
        translation: Vec3::new(0.5, 2.0, -1.0),
        rotation: v,
        ..Transform::identity()
    };
    let expected = Quat::from_rotation_z(7.0_f32.atan2(13.0)) * v;
    assert!((expected.dot(inner.rotation * v)).abs() < 0.999);
    probe(
        "mixed",
        &[outer, inner],
        child,
        expected,
        Vec3::new(1.0, -2.0, 3.0),
        24.0,
        true,
    );
}

#[test]
fn thin_nondegenerate_shear_without_collider_offset_still_exports_a_rigid_frame() {
    planar("thin", 1e-8, FRAC_PI_4, Vec3::ZERO);
}

#[test]
fn orthogonal_positive_and_reflected_frames_keep_the_existing_pose() {
    let q = Quat::from_rotation_z(0.37) * Quat::from_rotation_y(-0.41);
    let child = Transform {
        rotation: Quat::from_rotation_x(0.51),
        ..Transform::identity()
    };
    for (label, sx) in [("orthogonal", 2.0), ("orthogonal_mirror", -2.0)] {
        let parent = Transform {
            translation: Vec3::new(2.0, 3.0, 4.0),
            rotation: q,
            scale: Vec3::new(sx, 1.0, 1.0),
        };
        probe(
            label,
            &[parent],
            child,
            q * child.rotation,
            Vec3::X * 2.0,
            sx,
            false,
        );
    }
}

#[test]
fn ordinary_root_keeps_the_existing_pose() {
    let root = Transform {
        translation: Vec3::new(-1.0, 2.0, 4.0),
        rotation: Quat::from_rotation_y(0.43),
        ..Transform::identity()
    };
    probe("root", &[], root, root.rotation, Vec3::Z, 1.0, false);
}

fn planar(label: &str, sx: Real, angle: Real, center: Vec3) {
    let parent = Transform {
        translation: Vec3::new(1.0, 2.0, 3.0),
        scale: Vec3::new(sx, 1.0, 1.0),
        ..Transform::identity()
    };
    let child = Transform {
        translation: Vec3::X,
        rotation: Quat::from_rotation_y(FRAC_PI_4),
        ..Transform::identity()
    };
    probe(
        label,
        &[parent],
        child,
        Quat::from_rotation_y(angle),
        center,
        sx,
        true,
    );
}

fn probe(
    label: &str,
    ancestors: &[Transform],
    child: Transform,
    expected_rotation: Quat,
    offset: Vec3,
    expected_det: Real,
    sheared: bool,
) {
    let mut world = World::empty();
    let mut chain = Vec::new();
    for &authored in ancestors.iter().chain(std::iter::once(&child)) {
        let entity = world.spawn_node(NodeKind::Empty).unwrap();
        world.update_transform(entity, authored).unwrap();
        if let Some(&(parent, _)) = chain.last() {
            world.set_parent_checked(entity, Some(parent)).unwrap();
        }
        chain.push((entity, authored));
    }
    let entity = chain.last().unwrap().0;
    world
        .set_collider(
            entity,
            Some(ColliderComponent {
                shape: ColliderShape::Sphere { radius: 0.25 },
                local_transform: Transform {
                    translation: offset,
                    rotation: Quat::from_rotation_x(0.23),
                    scale: Vec3::new(1.0, 0.75, 1.25),
                },
                ..ColliderComponent::default()
            }),
        )
        .unwrap();
    let matrix = world.world_matrix(entity).unwrap();
    let projected = world.world_transform(entity).unwrap();
    let det = matrix.determinant();
    assert!((det / expected_det - 1.0).abs() < 1e-4);
    let [x, y, z] = [matrix.x_axis, matrix.y_axis, matrix.z_axis].map(|a| a.truncate().normalize());
    let shear = x.dot(y).abs().max(y.dot(z).abs()).max(z.dot(x).abs());
    if sheared {
        assert!(shear > 0.1);
    } else {
        assert!(shear < 1e-5);
    }
    // Independent point oracle applies each original local TRS in child->root order.
    // For the zero-offset case use a nonzero probe so this actually observes inherited shear.
    let point = if offset == Vec3::ZERO {
        Vec3::new(2.0, -1.0, 3.0)
    } else {
        offset
    };
    let expected_point = chain.iter().rev().fold(point, |p, (_, local)| {
        local.translation + local.rotation * (local.scale * p)
    });
    let affine_point = matrix.transform_point3(point);
    let point_error = affine_point.distance(expected_point);
    assert!(point_error < POINT_EPS);

    let handle = WorldHandle::new(1180);
    let collider_only = build_world_sync_state(handle, &world);
    assert_eq!(collider_only.colliders.len(), 1);
    assert!(collider_only.bodies.is_empty());
    world
        .set_rigid_body(entity, Some(RigidBodyComponent::default()))
        .unwrap();
    let both = build_world_sync_state(handle, &world);
    assert_eq!(both.world, handle);
    assert_eq!(
        both.bodies.len(),
        1,
        "invertible frame must not lose the body"
    );
    let body = &both.bodies[0];
    assert_eq!(body.entity, entity);
    assert_eq!(body.transform.translation, projected.translation);
    assert_eq!(body.transform.scale, projected.scale);
    assert_eq!(
        both.colliders, collider_only.colliders,
        "rigid projection stays out of collider geometry"
    );
    assert_eq!(both.materials, collider_only.materials);
    assert_eq!(world.world_matrix(entity), Some(matrix));
    assert_eq!(world.world_transform(entity), Some(projected));
    for &(node, local) in &chain {
        assert_eq!(world.local_transform(node), Some(local));
    }
    let q2 = body.transform.rotation.length_squared();
    assert!(q2.is_finite() && q2 > 0.5 && q2 < 1.5);
    if !sheared {
        assert_eq!(
            body.transform, projected,
            "orthogonal controls must be exact"
        );
    }
    // Normalize only this test copy so axes can be compared even for the old invalid body row.
    let observed = body.transform.rotation.normalize();
    let axis_error = [Vec3::X, Vec3::Y, Vec3::Z]
        .into_iter()
        .map(|axis| (observed * axis).distance(expected_rotation * axis))
        .fold(0.0, Real::max);
    println!("RIGID_FRAME label={label} det={det} shear={shear} q2={q2} axis_error={axis_error} point_error={point_error} point={:?} origin={:?} compact_scale={:?} collider_unchanged=true world_unchanged=true",
        affine_point.to_array(), body.transform.translation.to_array(), body.transform.scale.to_array());
    assert!((q2 - 1.0).abs() < UNIT_EPS && axis_error < AXIS_EPS,
        "scene physics must export an orthonormal rigid-body direction: {label} q2={q2} axis_error={axis_error}");
}
