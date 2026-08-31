//! Skeleton resource records.

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

#[derive(Clone, Serialize)]
struct AnimationSkeletonAssetRef<'a> {
    name: Option<&'a str>,
    bones: &'a [AnimationSkeletonBoneAsset],
}

impl<'a> From<&'a AnimationSkeletonAsset> for AnimationSkeletonAssetRef<'a> {
    fn from(value: &'a AnimationSkeletonAsset) -> Self {
        Self {
            name: value.name.as_deref(),
            bones: &value.bones,
        }
    }
}

impl AnimationSkeletonAsset {
    pub fn from_bytes(bytes: &[u8]) -> AnimationAssetResult<Self> {
        decode_binary_asset(AnimationBinaryAssetKind::Skeleton, bytes)
    }

    pub fn to_bytes(&self) -> AnimationAssetResult<Vec<u8>> {
        encode_binary_asset(
            AnimationBinaryAssetKind::Skeleton,
            &AnimationSkeletonAssetRef::from(self),
        )
    }
}

#[cfg(test)]
#[path = "skeleton/borrowed_encoding_tests.rs"]
mod borrowed_encoding_tests;
