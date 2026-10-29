#[path = "scene_system/component.rs"]
mod component;

use std::collections::BTreeMap;

use zircon_runtime::core::framework::scene::EntityId;
use zircon_runtime::core::framework::sound::SoundError;
use zircon_runtime::core::{CoreError, ModuleCleanupContext, ModuleLifecycle};
use zircon_runtime::plugin::{RuntimeExtensionRegistry, RuntimeExtensionRegistryError};
use zircon_runtime::scene::ecs::{RuntimeSceneSystemContext, SystemRef};
use zircon_runtime::scene::SystemStage;

use crate::module::DEFAULT_SOUND_MANAGER_NAME;
use crate::service_types::SceneSource;
use crate::DefaultSoundManager;

pub(crate) const PLAYBACK_MODULE: &str = "sound.playback.runtime";
pub(crate) const SYNC_SYSTEM: &str = "sound.playback.sync";

pub(crate) fn register_scene_system(
    registry: &mut RuntimeExtensionRegistry,
) -> Result<(), RuntimeExtensionRegistryError> {
    let owner = registry.intern_plugin_module(PLAYBACK_MODULE)?;
    registry
        .register_runtime_scene_system(owner, SYNC_SYSTEM, SystemStage::PostUpdate, || {
            let mut system = SoundSceneSystem::default();
            move |context| system.run(context)
        })
        .after(SystemRef::System(
            "zircon.scene.world_transform".to_string(),
        ))
        .register()
}

#[derive(Default)]
struct SoundSceneSystem {
    manager: Option<DefaultSoundManager>,
    sources: BTreeMap<EntityId, SceneSource>,
}

impl SoundSceneSystem {
    fn run(&mut self, context: RuntimeSceneSystemContext<'_>) -> Result<(), CoreError> {
        let manager = context
            .core
            .resolve_manager::<DefaultSoundManager>(DEFAULT_SOUND_MANAGER_NAME)?;
        if self
            .manager
            .as_ref()
            .is_some_and(|previous| !previous.same_session(&manager))
        {
            self.release();
        }
        self.manager = Some((*manager).clone());
        context.level.with_world(|world| {
            let removed = self
                .sources
                .keys()
                .copied()
                .filter(|entity| !component::enabled(world, *entity))
                .collect::<Vec<_>>();
            for entity in removed {
                if let Some(source) = self.sources.get(&entity) {
                    manager
                        .release_scene_source(source.id)
                        .map_err(|error| scene_error(entity, error))?;
                    self.sources.remove(&entity);
                }
            }
            for entity in world.stable_entity_ids() {
                if !component::enabled(world, entity) {
                    continue;
                }
                let result = component::read(world, entity).and_then(|descriptor| {
                    manager.sync_scene_source(self.sources.get(&entity), descriptor)
                });
                match result {
                    Ok(source) => {
                        self.sources.insert(entity, source);
                    }
                    Err(error) => {
                        if let Some(source) = self.sources.get(&entity) {
                            manager
                                .release_scene_source(source.id)
                                .map_err(|error| scene_error(entity, error))?;
                            self.sources.remove(&entity);
                        }
                        return Err(scene_error(entity, error));
                    }
                }
            }
            // Kira owns playback time. Scene ticks reconcile descriptors only;
            // they neither start an output device nor manually advance PCM.
            Ok(())
        })
    }

    fn release(&mut self) {
        if let Some(manager) = self.manager.take() {
            for source in std::mem::take(&mut self.sources).into_values() {
                let _ = manager.release_scene_source(source.id);
            }
        }
    }
}

impl Drop for SoundSceneSystem {
    fn drop(&mut self) {
        self.release();
    }
}

fn scene_error(entity: EntityId, error: SoundError) -> CoreError {
    CoreError::Initialization(
        SYNC_SYSTEM.to_string(),
        format!("audio entity {entity}: {error}"),
    )
}

pub(crate) struct SoundSceneLifecycle;

impl ModuleLifecycle for SoundSceneLifecycle {
    fn cleanup(&self, context: &ModuleCleanupContext<'_>) -> Result<(), CoreError> {
        context.with_existing_manager::<DefaultSoundManager>(
            DEFAULT_SOUND_MANAGER_NAME,
            |manager| {
                manager.release_scene_sources().map_err(|error| {
                    CoreError::Initialization(SYNC_SYSTEM.to_string(), error.to_string())
                })
            },
        )?;
        Ok(())
    }
}
