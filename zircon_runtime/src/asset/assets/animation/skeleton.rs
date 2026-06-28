use serde::{Deserialize, Serialize};

use super::binary::{decode_binary_asset, encode_binary_asset, AnimationBinaryAssetKind};
use super::error::AnimationAssetResult;
use crate::core::math::Real;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationSkeletonBoneAsset {
    pub name: String,
    pub parent_index: Option<u32>,
    pub local_translation: [Real; 3],
    pub local_rotation: [Real; 4],
    pub local_scale: [Real; 3],
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationSkeletonAsset {
    pub name: Option<String>,
    pub bones: Vec<AnimationSkeletonBoneAsset>,
}

impl AnimationSkeletonAsset {
    pub fn from_bytes(bytes: &[u8]) -> AnimationAssetResult<Self> {
        decode_binary_asset(AnimationBinaryAssetKind::Skeleton, bytes)
    }

    pub fn to_bytes(&self) -> AnimationAssetResult<Vec<u8>> {
        encode_binary_asset(AnimationBinaryAssetKind::Skeleton, self)
    }
}
