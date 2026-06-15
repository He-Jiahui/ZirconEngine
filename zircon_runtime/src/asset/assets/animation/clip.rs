use serde::{Deserialize, Serialize};

use super::binary::{
    decode_binary_asset_with_v1_payload_fallback, encode_binary_asset, AnimationBinaryAssetKind,
};
use super::channel::AnimationChannelAsset;
use super::reference::AnimationAssetReferenceBinary;
use crate::asset::AssetReference;
use crate::core::math::Real;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationClipBoneTrackAsset {
    pub bone_name: String,
    /// Stable optional retargeting id; v1 payloads continue to use `bone_name` alone.
    #[serde(default)]
    pub target_id: Option<String>,
    pub translation: AnimationChannelAsset,
    pub rotation: AnimationChannelAsset,
    pub scale: AnimationChannelAsset,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationEventTrackAsset {
    #[serde(default)]
    pub target_id: Option<String>,
    pub event: String,
    pub time_seconds: Real,
    #[serde(default)]
    pub payload: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationClipAsset {
    pub name: Option<String>,
    pub skeleton: AssetReference,
    pub duration_seconds: Real,
    pub tracks: Vec<AnimationClipBoneTrackAsset>,
    #[serde(default)]
    pub event_tracks: Vec<AnimationEventTrackAsset>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct AnimationClipBinaryAsset {
    name: Option<String>,
    skeleton: AnimationAssetReferenceBinary,
    duration_seconds: Real,
    tracks: Vec<AnimationClipBoneTrackAsset>,
    #[serde(default)]
    event_tracks: Vec<AnimationEventTrackAsset>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct AnimationClipBinaryAssetV1 {
    name: Option<String>,
    skeleton: AnimationAssetReferenceBinary,
    duration_seconds: Real,
    tracks: Vec<AnimationClipBoneTrackAssetV1>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct AnimationClipBoneTrackAssetV1 {
    bone_name: String,
    translation: AnimationChannelAsset,
    rotation: AnimationChannelAsset,
    scale: AnimationChannelAsset,
}

impl From<&AnimationClipAsset> for AnimationClipBinaryAsset {
    fn from(value: &AnimationClipAsset) -> Self {
        Self {
            name: value.name.clone(),
            skeleton: AnimationAssetReferenceBinary::from(&value.skeleton),
            duration_seconds: value.duration_seconds,
            tracks: value.tracks.clone(),
            event_tracks: value.event_tracks.clone(),
        }
    }
}

impl TryFrom<AnimationClipBinaryAsset> for AnimationClipAsset {
    type Error = String;

    fn try_from(value: AnimationClipBinaryAsset) -> Result<Self, Self::Error> {
        Ok(Self {
            name: value.name,
            skeleton: value.skeleton.try_into()?,
            duration_seconds: value.duration_seconds,
            tracks: value.tracks,
            event_tracks: value.event_tracks,
        })
    }
}

impl TryFrom<AnimationClipBinaryAssetV1> for AnimationClipBinaryAsset {
    type Error = String;

    fn try_from(value: AnimationClipBinaryAssetV1) -> Result<Self, Self::Error> {
        Ok(Self {
            name: value.name,
            skeleton: value.skeleton,
            duration_seconds: value.duration_seconds,
            tracks: value
                .tracks
                .into_iter()
                .map(|track| AnimationClipBoneTrackAsset {
                    bone_name: track.bone_name,
                    target_id: None,
                    translation: track.translation,
                    rotation: track.rotation,
                    scale: track.scale,
                })
                .collect(),
            event_tracks: Vec::new(),
        })
    }
}

impl TryFrom<AnimationClipBinaryAssetV1> for AnimationClipAsset {
    type Error = String;

    fn try_from(value: AnimationClipBinaryAssetV1) -> Result<Self, Self::Error> {
        Ok(Self {
            name: value.name,
            skeleton: value.skeleton.try_into()?,
            duration_seconds: value.duration_seconds,
            tracks: value
                .tracks
                .into_iter()
                .map(|track| AnimationClipBoneTrackAsset {
                    bone_name: track.bone_name,
                    target_id: None,
                    translation: track.translation,
                    rotation: track.rotation,
                    scale: track.scale,
                })
                .collect(),
            event_tracks: Vec::new(),
        })
    }
}

impl AnimationClipAsset {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        decode_binary_asset_with_v1_payload_fallback::<
            AnimationClipBinaryAsset,
            AnimationClipBinaryAssetV1,
        >(AnimationBinaryAssetKind::Clip, bytes)
        .and_then(AnimationClipAsset::try_from)
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        encode_binary_asset(
            AnimationBinaryAssetKind::Clip,
            &AnimationClipBinaryAsset::from(self),
        )
    }

    pub fn direct_references(&self) -> Vec<AssetReference> {
        vec![self.skeleton.clone()]
    }
}
