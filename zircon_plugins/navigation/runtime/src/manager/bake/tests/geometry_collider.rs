use super::*;

fn seeded_geometry() -> BakeGeometry {
    BakeGeometry {
        vertices: vec![[1.0, 2.0, 3.0], [2.0, 2.0, 3.0], [1.0, 3.0, 3.0]],
        indices: vec![0, 1, 2],
        triangle_areas: vec![17],
        source_entities: 4,
        skipped_navigation_components: 5,
        removed_by_modifier: 6,
        modified_by_area_override: 7,
        carved_by_obstacle: 8,
        render_mesh_fallback: true,
        unbound_render_nodes: 9,
        render_lod_levels_not_selected: 10,
    }
}

fn assert_rejects_nonfinite_shape(shape: ColliderShape, label: &str) {
    let mut geometry = seeded_geometry();
    let before = geometry.clone();
    let error = collect_collider_shape_geometry(Mat4::IDENTITY, &shape, &mut geometry, 3)
        .expect_err("non-finite collider data must fail before Recast");

    assert!(
        error.to_string().contains("non-finite"),
        "{label} should report a finite-input error, got {error}"
    );
    assert_eq!(
        geometry, before,
        "{label} must not partially append geometry"
    );
}

#[test]
fn rejects_nonfinite_box_sphere_capsule_cylinder_and_convex_hull_inputs() {
    assert_rejects_nonfinite_shape(
        ColliderShape::Box {
            half_extents: Vec3::new(f32::NAN, 1.0, 1.0),
        },
        "box NaN",
    );
    assert_rejects_nonfinite_shape(
        ColliderShape::Sphere {
            radius: f32::INFINITY,
        },
        "sphere infinity",
    );
    assert_rejects_nonfinite_shape(
        ColliderShape::Capsule {
            radius: 0.5,
            half_height: f32::NAN,
        },
        "capsule NaN",
    );
    assert_rejects_nonfinite_shape(
        ColliderShape::Cylinder {
            radius: f32::INFINITY,
            half_height: 1.0,
        },
        "cylinder infinity",
    );
    assert_rejects_nonfinite_shape(
        ColliderShape::ConvexHull {
            points: vec![Vec3::ZERO, Vec3::new(f32::NAN, 1.0, 0.0)],
        },
        "convex hull NaN",
    );
}

#[test]
fn rejects_nonfinite_compound_child_transform_and_shape() {
    assert_rejects_nonfinite_shape(
        ColliderShape::Compound {
            children: vec![(
                Transform::from_translation(Vec3::new(f32::NAN, 0.0, 0.0)),
                Box::new(ColliderShape::Box {
                    half_extents: Vec3::ONE,
                }),
            )],
        },
        "compound child transform NaN",
    );
    assert_rejects_nonfinite_shape(
        ColliderShape::Compound {
            children: vec![
                (
                    Transform::identity(),
                    Box::new(ColliderShape::Box {
                        half_extents: Vec3::ONE,
                    }),
                ),
                (
                    Transform::identity(),
                    Box::new(ColliderShape::Sphere {
                        radius: f32::INFINITY,
                    }),
                ),
            ],
        },
        "compound child sphere infinity",
    );
}

#[test]
fn rejects_nonfinite_transformed_vertices_without_changing_existing_geometry() {
    let mut geometry = seeded_geometry();
    let before = geometry.clone();
    let finite_but_overflowing_matrix = Mat4::from_scale(Vec3::splat(f32::MAX));
    let error = collect_collider_shape_geometry(
        finite_but_overflowing_matrix,
        &ColliderShape::Box {
            half_extents: Vec3::splat(2.0),
        },
        &mut geometry,
        3,
    )
    .expect_err("overflowed transformed collider vertices must fail closed");

    assert!(error.to_string().contains("transformed vertex"));
    assert_eq!(geometry, before);
}

#[test]
fn finite_collider_geometry_preserves_existing_data_and_rebases_indices() {
    let mut geometry = seeded_geometry();
    collect_collider_shape_geometry(
        Mat4::IDENTITY,
        &ColliderShape::Box {
            half_extents: Vec3::splat(0.5),
        },
        &mut geometry,
        23,
    )
    .expect("finite collider geometry should be accepted");

    assert_eq!(geometry.vertices.len(), 7);
    assert_eq!(&geometry.indices[..3], &[0, 1, 2]);
    assert_eq!(&geometry.indices[3..], &[3, 4, 5, 3, 5, 6]);
    assert_eq!(&geometry.triangle_areas[..1], &[17]);
    assert_eq!(&geometry.triangle_areas[1..], &[23, 23]);
    assert!(geometry
        .indices
        .iter()
        .all(|index| (*index as usize) < geometry.vertices.len()));
}
