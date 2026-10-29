use std::collections::BTreeSet;

use zircon_runtime::core::framework::scene::EntityId;
use zircon_runtime::core::framework::sound::{
    SoundError, SoundPlayRequests, SoundPlaybackManager, SoundSourceDescriptor, SoundSourceManager,
    AUDIO_SOURCE_COMPONENT_TYPE,
};
use zircon_runtime::core::CoreError;
use zircon_runtime::scene::World;

use super::{scene_error, SoundSceneSystem, SYNC_SYSTEM};
use crate::DefaultSoundManager;

impl SoundSceneSystem {
    pub(super) fn consume_requests(
        &mut self,
        manager: &DefaultSoundManager,
        world: &mut World,
    ) -> Result<BTreeSet<EntityId>, CoreError> {
        let requests = world
            .get_resource_mut::<SoundPlayRequests>()
            .map(|requests| std::mem::take(&mut requests.0))
            .unwrap_or_default();
        let mut retriggered = BTreeSet::new();
        for request in requests {
            let entity = request.entity;
            // Removed/inactive entities cannot start deferred playback.
            if world.active_in_hierarchy(entity) != Some(true) {
                continue;
            }
            let Some(transform) = world.world_transform(entity) else {
                continue;
            };
            let position = transform.translation.to_array();
            let clip = manager
                .load_clip(&request.uri)
                .map_err(|error| scene_error(entity, error))?;
            if let Some(mut value) = world
                .dynamic_component(entity, AUDIO_SOURCE_COMPONENT_TYPE)
                .cloned()
            {
                let fields = value.as_object_mut().ok_or_else(|| {
                    scene_error(
                        entity,
                        SoundError::InvalidParameter("AudioSource must be an object".to_string()),
                    )
                })?;
                fields.insert("input".to_string(), serde_json::json!({"Clip": clip}));
                fields.insert("playing".to_string(), true.into());
                fields.insert("enabled".to_string(), true.into());
                fields.insert("position".to_string(), serde_json::json!(position));
                // Preserve authored gain, loop, routing and spatial settings;
                // use the registered dynamic-component API and its normal gate.
                world
                    .set_dynamic_component(entity, AUDIO_SOURCE_COMPONENT_TYPE, value)
                    .map_err(|error| {
                        CoreError::Initialization(SYNC_SYSTEM.to_string(), error.to_string())
                    })?;
                retriggered.insert(entity);
            } else {
                // No component is synthesized. Each call owns one source at
                // the emission position, with the existing service defaults.
                let mut descriptor = SoundSourceDescriptor::clip(clip);
                descriptor.position = position;
                let source = manager
                    .sync_scene_source(None, descriptor)
                    .map_err(|error| scene_error(entity, error))?;
                self.transients.insert(source.id, entity);
            }
        }
        Ok(retriggered)
    }

    pub(super) fn release_finished_transients(
        &mut self,
        manager: &DefaultSoundManager,
        world: &World,
    ) -> Result<(), CoreError> {
        let mut released = Vec::new();
        for (&source, &entity) in &self.transients {
            let finished = if world.active_in_hierarchy(entity) != Some(true) {
                true
            } else {
                match manager.source_status(source) {
                    Ok(_) => false,
                    Err(SoundError::UnknownSource { .. }) => true,
                    Err(error) => return Err(scene_error(entity, error)),
                }
            };
            if finished {
                manager
                    .release_scene_source(source)
                    .map_err(|error| scene_error(entity, error))?;
                released.push(source);
            }
        }
        for source in released {
            self.transients.remove(&source);
        }
        Ok(())
    }
}
