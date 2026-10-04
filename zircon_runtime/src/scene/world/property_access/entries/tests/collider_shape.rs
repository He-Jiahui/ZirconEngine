use crate::core::math::{Transform, Vec3};
use crate::core::resource::{AssetReference, ResourceLocator};

use super::*;

#[test]
fn extended_collider_shape_projection_matches_capacity_and_preserves_shape_data() {
    let mesh = AssetReference::from_locator(
        ResourceLocator::parse("res://physics/property_projection.physics_mesh").unwrap(),
    );
    let shapes = [
        ColliderShape::Cylinder {
            radius: 0.75,
            half_height: 1.25,
        },
        ColliderShape::ConvexHull {
            points: vec![Vec3::ZERO, Vec3::X, Vec3::Y, Vec3::Z],
        },
        ColliderShape::TriangleMesh { mesh: mesh.clone() },
        ColliderShape::HeightField {
            resolution: [8, 4],
            heights: mesh,
        },
        ColliderShape::Compound {
            children: vec![(
                Transform::from_translation(Vec3::new(1.0, 2.0, 3.0)),
                Box::new(ColliderShape::Sphere { radius: 0.5 }),
            )],
        },
    ];

    for shape in shapes {
        let mut entries = Vec::new();
        assert!(visit_collider_shape_property_entries(
            &shape,
            "Collider.shape",
            &mut |path, value, animatable| {
                entries.push((path.to_string(), value(), animatable));
                true
            },
        ));
        assert_eq!(
            entries.len(),
            collider_shape_property_entry_capacity(&shape)
        );
        assert!(entries
            .iter()
            .any(|(path, _, _)| path == "Collider.shape.kind"));
    }
}

#[test]
fn compound_projection_keeps_child_transform_and_shape_fields() {
    let compound = ColliderShape::Compound {
        children: vec![(
            Transform::from_translation(Vec3::new(1.0, 2.0, 3.0)),
            Box::new(ColliderShape::Sphere { radius: 0.5 }),
        )],
    };
    let mut entries = Vec::new();
    assert!(visit_collider_shape_property_entries(
        &compound,
        "Collider.shape",
        &mut |path, value, _| {
            entries.push((path.to_string(), value()));
            true
        },
    ));
    assert!(entries.iter().any(|(path, value)| {
        path == "Collider.shape.children.0.transform.translation"
            && *value == ScenePropertyValue::Vec3([1.0, 2.0, 3.0])
    }));
    assert!(entries.iter().any(|(path, value)| {
        path == "Collider.shape.children.0.shape.radius"
            && *value == ScenePropertyValue::Scalar(0.5)
    }));
}
