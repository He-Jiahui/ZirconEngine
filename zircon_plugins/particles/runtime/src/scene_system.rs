#[path = "scene_system/component.rs"]
mod component;

use std::collections::BTreeMap;

use zircon_runtime::core::framework::scene::EntityId;
use zircon_runtime::core::{CoreError, ModuleCleanupContext, ModuleLifecycle};
use zircon_runtime::plugin::{RuntimeExtensionRegistry, RuntimeExtensionRegistryError};
use zircon_runtime::scene::ecs::{RuntimeSceneSystemContext, SystemRef};
use zircon_runtime::scene::SystemStage;

use crate::{ParticleEmitterHandle, ParticlesManager, PARTICLES_MANAGER_NAME};

pub(crate) const SIMULATION_MODULE: &str = "particles.simulation.runtime";
pub(crate) const TICK_SYSTEM: &str = "particles.simulation.tick";

pub(crate) fn register_scene_system(
    registry: &mut RuntimeExtensionRegistry,
) -> Result<(), RuntimeExtensionRegistryError> {
    let owner = registry.intern_plugin_module(SIMULATION_MODULE)?;
    registry
        .register_runtime_scene_system(owner, TICK_SYSTEM, SystemStage::PostUpdate, || {
            let mut system = ParticleSceneSystem::default();
            move |context| system.run(context)
        })
        .after(SystemRef::System(
            "zircon.scene.world_transform".to_string(),
        ))
        .register()
}

#[derive(Default)]
struct ParticleSceneSystem {
    manager: Option<ParticlesManager>,
    handles: BTreeMap<EntityId, ParticleEmitterHandle>,
}

impl ParticleSceneSystem {
    fn run(&mut self, context: RuntimeSceneSystemContext<'_>) -> Result<(), CoreError> {
        let manager = context
            .core
            .resolve_manager::<ParticlesManager>(PARTICLES_MANAGER_NAME)?;
        if self
            .manager
            .as_ref()
            .is_some_and(|previous| !previous.same_session(&manager))
        {
            self.release();
        }
        self.manager = Some((*manager).clone());
        context.level.with_world(|world| {
            self.handles.retain(|entity, handle| {
                if component::enabled(world, *entity) {
                    true
                } else {
                    let _ = manager.remove(*handle);
                    false
                }
            });
            for entity in world.stable_entity_ids() {
                if !component::enabled(world, entity) {
                    continue;
                }
                let result = component::read(world, entity).and_then(|component| {
                    manager.tick_scene_emitter(
                        self.handles.get(&entity).copied(),
                        component,
                        context.tick().delta_seconds(),
                    )
                });
                match result {
                    Ok(handle) => {
                        self.handles.insert(entity, handle);
                    }
                    Err(error) => {
                        if let Some(handle) = self.handles.remove(&entity) {
                            let _ = manager.remove(handle);
                        }
                        return Err(CoreError::Initialization(
                            TICK_SYSTEM.to_string(),
                            format!("particle entity {entity}: {error}"),
                        ));
                    }
                }
            }
            Ok(())
        })
    }

    fn release(&mut self) {
        if let Some(manager) = self.manager.take() {
            for handle in std::mem::take(&mut self.handles).into_values() {
                let _ = manager.remove(handle);
            }
        }
    }
}

impl Drop for ParticleSceneSystem {
    fn drop(&mut self) {
        self.release();
    }
}

pub(crate) struct ParticlesSceneLifecycle;

impl ModuleLifecycle for ParticlesSceneLifecycle {
    fn cleanup(&self, context: &ModuleCleanupContext<'_>) -> Result<(), CoreError> {
        context.with_existing_manager::<ParticlesManager>(PARTICLES_MANAGER_NAME, |manager| {
            manager.release_scene_emitters();
            Ok(())
        })?;
        Ok(())
    }
}
