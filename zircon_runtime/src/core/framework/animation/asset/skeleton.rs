//! Skeleton resource records.

use serde::{Deserialize, Deserializer, Serialize};

use super::binary::{decode_binary_asset, encode_binary_asset, AnimationBinaryAssetKind};
use super::error::{AnimationAssetError, AnimationAssetResult};
use crate::core::math::Real;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationSkeletonBoneAsset {
    pub name: String,
    pub parent_index: Option<u32>,
    pub local_translation: [Real; 3],
    pub local_rotation: [Real; 4],
    pub local_scale: [Real; 3],
}

/// 片段采样与运行时目标表共享的骨架资源；骨骼顺序是姿态槽位的依据，
/// 引用它的片段需在资源加载后再做目标解析。
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AnimationSkeletonAsset {
    pub name: Option<String>,
    pub bones: Vec<AnimationSkeletonBoneAsset>,
}

#[derive(Deserialize)]
struct AnimationSkeletonAssetDocument {
    name: Option<String>,
    bones: Vec<AnimationSkeletonBoneAsset>,
}

impl<'de> Deserialize<'de> for AnimationSkeletonAsset {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let document = AnimationSkeletonAssetDocument::deserialize(deserializer)?;
        let asset = Self {
            name: document.name,
            bones: document.bones,
        };
        asset
            .validate_hierarchy()
            .map_err(serde::de::Error::custom)?;
        Ok(asset)
    }
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
    fn validate_hierarchy(&self) -> Result<(), String> {
        let mut state = vec![0_u8; self.bones.len()];
        for start in 0..self.bones.len() {
            if state[start] == 2 {
                continue;
            }
            let mut path = Vec::new();
            let mut current = start;
            loop {
                match state[current] {
                    1 => return Err(format!("bone parent cycle reaches index {current}")),
                    2 => break,
                    _ => {}
                }
                state[current] = 1;
                path.push(current);
                let Some(parent) = self.bones[current].parent_index else {
                    break;
                };
                current = usize::try_from(parent)
                    .ok()
                    .filter(|&parent| parent < self.bones.len())
                    .ok_or_else(|| {
                        format!("bone index {current} has out-of-range parent index {parent}")
                    })?;
            }
            for index in path {
                state[index] = 2;
            }
        }
        Ok(())
    }

    pub fn from_bytes(bytes: &[u8]) -> AnimationAssetResult<Self> {
        decode_binary_asset(AnimationBinaryAssetKind::Skeleton, bytes)
    }

    pub fn to_bytes(&self) -> AnimationAssetResult<Vec<u8>> {
        self.validate_hierarchy()
            .map_err(|reason| AnimationAssetError::InvalidSkeletonHierarchy { reason })?;
        encode_binary_asset(
            AnimationBinaryAssetKind::Skeleton,
            &AnimationSkeletonAssetRef::from(self),
        )
    }
}

#[cfg(test)]
#[path = "../../../animation/asset/skeleton/tests/borrowed_encoding.rs"]
mod borrowed_encoding_tests;

#[cfg(test)]
#[path = "../../../animation/asset/skeleton/tests/hierarchy_validation.rs"]
mod hierarchy_validation_tests;
