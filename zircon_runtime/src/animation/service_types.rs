use crate::core::framework::animation::{AnimationPlaybackSettings, AnimationTrackPath};

use super::AnimationInterface;

#[derive(Clone, Debug, Default)]
pub struct AnimationDriver;

#[derive(Clone, Debug, Default)]
pub struct DefaultAnimationManager {
    playback_settings: AnimationPlaybackSettings,
}

impl crate::core::framework::animation::AnimationManager for DefaultAnimationManager {
    fn playback_settings(&self) -> AnimationPlaybackSettings {
        self.playback_settings.clone()
    }

    fn normalize_track_path(&self, path: &AnimationTrackPath) -> AnimationTrackPath {
        path.clone()
    }
}

impl AnimationInterface for DefaultAnimationManager {}
