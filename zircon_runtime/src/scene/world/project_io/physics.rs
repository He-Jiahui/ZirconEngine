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
    }
}
