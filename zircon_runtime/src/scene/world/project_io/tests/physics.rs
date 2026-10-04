use crate::asset::{AssetReference, TransformAsset};
use crate::core::resource::ResourceLocator;

use super::*;

#[test]
fn extended_collider_shapes_round_trip_through_scene_project_io() {
    let mesh = AssetReference::from_locator(
        ResourceLocator::parse("res://physics/project_io.physics_mesh").unwrap(),
    );
    let shapes = [
        SceneColliderShapeAsset::Cylinder {
            radius: 0.75,
            half_height: 1.25,
        },
        SceneColliderShapeAsset::ConvexHull {
            points: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
            ],
        },
        SceneColliderShapeAsset::TriangleMesh { mesh: mesh.clone() },
        SceneColliderShapeAsset::HeightField {
            resolution: [8, 4],
            heights: mesh,
        },
        SceneColliderShapeAsset::Compound {
            children: vec![(
                TransformAsset {
                    translation: [1.0, 2.0, 3.0],
                    rotation: [0.0, 0.0, 0.0, 1.0],
                    scale: [2.0, 2.0, 2.0],
                },
                Box::new(SceneColliderShapeAsset::Sphere { radius: 0.5 }),
            )],
        },
    ];

    for shape in shapes {
        let runtime_shape = collider_shape_from_asset(shape.clone());
        assert_eq!(collider_shape_to_asset(runtime_shape), shape);
    }
}
