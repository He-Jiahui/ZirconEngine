use crate::asset::assets::SceneColliderShapeAsset;
use crate::scene::components::ColliderShape;
// 项目磁盘形状到运行时碰撞体的边界转换；保持资源引用与数值字段的往返语义。
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
#[path = "tests/physics.rs"]
mod tests;
