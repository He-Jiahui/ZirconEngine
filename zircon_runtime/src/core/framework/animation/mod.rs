//! Animation framework contracts (playback settings, track paths, parameters).

use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnimationParameterValue {
    Bool(bool),
    Integer(i32),
    Scalar(f32),
    Vec2([f32; 2]),
    Vec3([f32; 3]),
    Vec4([f32; 4]),
    Trigger,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AnimationTrackPath {
    raw: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AnimationTrackPathError;

impl fmt::Display for AnimationTrackPathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid animation track path")
    }
}

impl std::error::Error for AnimationTrackPathError {}

impl AnimationTrackPath {
    pub fn new(
        entity_path: super::scene::EntityPath,
        property_path: super::scene::ComponentPropertyPath,
    ) -> Self {
        Self {
            raw: format!("{entity_path}:{property_path}"),
        }
    }

    pub fn parse(raw: &str) -> Result<Self, AnimationTrackPathError> {
        let (entity_path, property_path) = raw.split_once(':').ok_or(AnimationTrackPathError)?;
        let entity_path = super::scene::EntityPath::parse(entity_path)
            .map_err(|_| AnimationTrackPathError)?;
        let property_path = super::scene::ComponentPropertyPath::parse(property_path)
            .map_err(|_| AnimationTrackPathError)?;
        Ok(Self {
            raw: format!("{entity_path}:{property_path}"),
        })
    }

    pub fn as_str(&self) -> &str {
        &self.raw
    }
}

impl fmt::Display for AnimationTrackPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.raw)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationPlaybackSettings {
    pub enabled: bool,
    pub property_tracks: bool,
    pub skeletal_clips: bool,
    pub graphs: bool,
    pub state_machines: bool,
}

impl Default for AnimationPlaybackSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            property_tracks: true,
            skeletal_clips: true,
            graphs: true,
            state_machines: true,
        }
    }
}

pub trait AnimationManager: Send + Sync {
    fn playback_settings(&self) -> AnimationPlaybackSettings;
    fn normalize_track_path(&self, path: &AnimationTrackPath) -> AnimationTrackPath;
}
