use crate::asset::AnimationSequenceAsset;
use crate::core::framework::animation::{
    AnimationManager, AnimationPlaybackSettings, AnimationTrackPath,
};
use crate::core::math::Real;
use crate::core::CoreError;
use crate::scene::world::World;

use super::{apply_sequence_to_world, AnimationSequenceApplyReport};

pub trait AnimationInterface: AnimationManager {
    fn store_playback_settings(
        &self,
        playback_settings: AnimationPlaybackSettings,
    ) -> Result<(), CoreError>;

    fn apply_sequence(
        &self,
        world: &mut World,
        sequence: &AnimationSequenceAsset,
        time_seconds: Real,
    ) -> Result<AnimationSequenceApplyReport, String> {
        apply_sequence_to_world(world, sequence, time_seconds)
    }

    fn canonical_track_path(
        &self,
        world: &World,
        path: &AnimationTrackPath,
    ) -> Result<AnimationTrackPath, String> {
        let (entity_path, property_path) = path.split().map_err(|error| error.to_string())?;
        let entity = world
            .resolve_entity_path(&entity_path)
            .ok_or_else(|| format!("unknown animation entity path `{entity_path}`"))?;
        let canonical_entity_path = world
            .entity_path(entity)
            .ok_or_else(|| format!("missing canonical entity path for {entity}"))?;
        world.property(entity, &property_path)?;
        Ok(AnimationTrackPath::new(
            canonical_entity_path,
            property_path,
        ))
    }

    fn is_enabled(&self) -> bool {
        self.playback_settings().enabled
    }
}
