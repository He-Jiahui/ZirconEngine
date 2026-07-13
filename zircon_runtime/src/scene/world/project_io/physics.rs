use crate::asset::assets::SceneColliderShapeAsset;
use crate::scene::components::ColliderShape;
pub(super) fn collider_shape_from_asset(shape: SceneColliderShapeAsset) -> ColliderShape {
    match shape {
        SceneColliderShapeAsset::Box { half_extents } => ColliderShape::Box {
            half_extents: crate::core::math::Vec3::from_array(half_extents),
        },
        SceneColliderShapeAsset::Sphere { radius } => ColliderShape::Sphere { radius },
        SceneColliderShapeAsset::Capsule {
            radius,
            half_height,
        } => ColliderShape::Capsule {
            radius,
            half_height,
        },
        SceneColliderShapeAsset::Cylinder {
            radius,
            half_height,
        } => ColliderShape::Cylinder {
            radius,
            half_height,
        },
        SceneColliderShapeAsset::ConvexHull { points } => ColliderShape::ConvexHull {
            points: points
                .into_iter()
                .map(crate::core::math::Vec3::from_array)
                .collect(),
        },
        SceneColliderShapeAsset::TriangleMesh { mesh } => ColliderShape::TriangleMesh { mesh },
        SceneColliderShapeAsset::HeightField {
            resolution,
            heights,
        } => ColliderShape::HeightField {
            resolution,
            heights,
        },
        SceneColliderShapeAsset::Compound { children } => ColliderShape::Compound {
            children: children
                .into_iter()
                .map(|(transform, shape)| {
                    (
                        crate::core::math::Transform {
                            translation: crate::core::math::Vec3::from_array(transform.translation),
                            rotation: crate::core::math::Quat::from_array(transform.rotation),
                            scale: crate::core::math::Vec3::from_array(transform.scale),
                        },
                        Box::new(collider_shape_from_asset(*shape)),
                    )
                })
                .collect(),
        },
    }
}

pub(super) fn collider_shape_to_asset(shape: ColliderShape) -> SceneColliderShapeAsset {
    match shape {
        ColliderShape::Box { half_extents } => SceneColliderShapeAsset::Box {
            half_extents: half_extents.to_array(),
        },
        ColliderShape::Sphere { radius } => SceneColliderShapeAsset::Sphere { radius },
        ColliderShape::Capsule {
            radius,
            half_height,
        } => SceneColliderShapeAsset::Capsule {
            radius,
            half_height,
        },
        ColliderShape::Cylinder {
            radius,
            half_height,
        } => SceneColliderShapeAsset::Cylinder {
            radius,
            half_height,
        },
        ColliderShape::ConvexHull { points } => SceneColliderShapeAsset::ConvexHull {
            points: points.into_iter().map(|point| point.to_array()).collect(),
        },
        ColliderShape::TriangleMesh { mesh } => SceneColliderShapeAsset::TriangleMesh { mesh },
        ColliderShape::HeightField {
            resolution,
            heights,
        } => SceneColliderShapeAsset::HeightField {
            resolution,
            heights,
        },
        ColliderShape::Compound { children } => SceneColliderShapeAsset::Compound {
            children: children
                .into_iter()
                .map(|(transform, shape)| {
                    (
                        crate::asset::TransformAsset {
                            translation: transform.translation.to_array(),
                            rotation: transform.rotation.to_array(),
                            scale: transform.scale.to_array(),
                        },
                        Box::new(collider_shape_to_asset(*shape)),
                    )
                })
                .collect(),
        },
    }
}

#[cfg(test)]
mod tests {
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
}
