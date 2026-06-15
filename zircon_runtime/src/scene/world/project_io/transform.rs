use crate::asset::assets::TransformAsset;
pub(super) fn transform_from_asset(transform: TransformAsset) -> crate::core::math::Transform {
    crate::core::math::Transform {
        translation: crate::core::math::Vec3::from_array(transform.translation),
        rotation: crate::core::math::Quat::from_array(transform.rotation),
        scale: crate::core::math::Vec3::from_array(transform.scale),
    }
}

pub(super) fn transform_to_asset(transform: crate::core::math::Transform) -> TransformAsset {
    TransformAsset {
        translation: transform.translation.to_array(),
        rotation: transform.rotation.to_array(),
        scale: transform.scale.to_array(),
    }
}
