//! Timeline sequence resource records.

use serde::{Deserialize, Serialize};

use super::binary::{
    decode_binary_asset_with_v1_payload_fallback, encode_binary_asset, AnimationBinaryAssetKind,
};
use super::channel::AnimationChannelAsset;
use super::error::{AnimationAssetError, AnimationAssetResult};
use crate::core::framework::animation::AnimationTrackPath;
use crate::core::framework::scene::{ComponentPropertyPath, EntityPath};
use crate::core::math::Real;

#[cfg(test)]
#[path = "sequence/borrowed_encoding_tests.rs"]
mod borrowed_encoding_tests;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationSequenceTrackAsset {
    pub property_path: ComponentPropertyPath,
    pub channel: AnimationChannelAsset,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationSequenceBindingAsset {
    pub entity_path: EntityPath,
    /// Stable optional scene target id; `entity_path` remains the path fallback for existing assets.
    #[serde(default)]
    pub target_id: Option<String>,
    pub tracks: Vec<AnimationSequenceTrackAsset>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct AnimationSequenceAssetV1 {
    name: Option<String>,
    duration_seconds: Real,
    frames_per_second: Real,
    bindings: Vec<AnimationSequenceBindingAssetV1>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct AnimationSequenceBindingAssetV1 {
    entity_path: EntityPath,
    tracks: Vec<AnimationSequenceTrackAsset>,
}

impl TryFrom<AnimationSequenceAssetV1> for AnimationSequenceAsset {
    type Error = AnimationAssetError;

    fn try_from(value: AnimationSequenceAssetV1) -> Result<Self, Self::Error> {
        Ok(Self {
            name: value.name,
            duration_seconds: value.duration_seconds,
            frames_per_second: value.frames_per_second,
            bindings: value
                .bindings
                .into_iter()
                .map(|binding| AnimationSequenceBindingAsset {
                    entity_path: binding.entity_path,
                    target_id: None,
                    tracks: binding.tracks,
                })
                .collect(),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationSequenceAsset {
    pub name: Option<String>,
    pub duration_seconds: Real,
    pub frames_per_second: Real,
    pub bindings: Vec<AnimationSequenceBindingAsset>,
}

#[derive(Clone, Serialize)]
struct AnimationSequenceAssetRef<'a> {
    name: &'a Option<String>,
    duration_seconds: Real,
    frames_per_second: Real,
    bindings: &'a [AnimationSequenceBindingAsset],
}

impl<'a> From<&'a AnimationSequenceAsset> for AnimationSequenceAssetRef<'a> {
    fn from(value: &'a AnimationSequenceAsset) -> Self {
        Self {
            name: &value.name,
            duration_seconds: value.duration_seconds,
            frames_per_second: value.frames_per_second,
            bindings: &value.bindings,
        }
    }
}

impl AnimationSequenceAsset {
    pub fn from_bytes(bytes: &[u8]) -> AnimationAssetResult<Self> {
        decode_binary_asset_with_v1_payload_fallback::<
            AnimationSequenceAsset,
            AnimationSequenceAssetV1,
        >(AnimationBinaryAssetKind::Sequence, bytes)
    }

    pub fn to_bytes(&self) -> AnimationAssetResult<Vec<u8>> {
        encode_binary_asset(
            AnimationBinaryAssetKind::Sequence,
            &AnimationSequenceAssetRef::from(self),
        )
    }

    pub fn track_paths(&self) -> Vec<AnimationTrackPath> {
        self.bindings
            .iter()
            .flat_map(|binding| {
                binding.tracks.iter().cloned().map(|track| {
                    AnimationTrackPath::new(binding.entity_path.clone(), track.property_path)
                })
            })
            .collect()
    }

    pub fn target_track_paths(&self) -> Vec<(Option<String>, AnimationTrackPath)> {
        self.bindings
            .iter()
            .flat_map(|binding| {
                binding.tracks.iter().cloned().map(|track| {
                    (
                        binding.target_id.clone(),
                        AnimationTrackPath::new(binding.entity_path.clone(), track.property_path),
                    )
                })
            })
            .collect()
    }
}
