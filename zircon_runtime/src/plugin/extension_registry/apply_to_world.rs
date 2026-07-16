mod component;

use crate::plugin::{RuntimeExtensionRegistry, RuntimeExtensionRegistryError};
use crate::scene::{
    World, WorldRuntimeExtensionError, WorldRuntimeExtensionPlan, WorldRuntimeExtensionRegistration,
};

impl RuntimeExtensionRegistry {
    pub fn apply_to_world(
        &mut self,
        world: &mut World,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        self.finalize();
        self.world_runtime_extension_plan()?
            .apply_to_world(world)
            .map_err(|error| RuntimeExtensionRegistryError::WorldRegistration(error.to_string()))
    }

    pub fn world_runtime_extension_plan(
        &self,
    ) -> Result<WorldRuntimeExtensionPlan, RuntimeExtensionRegistryError> {
        let mut registrations = Vec::new();
        for component in self.components().iter().cloned() {
            let key = format!("component:{}", component.type_id);
            let apply_key = key.clone();
            registrations.push(WorldRuntimeExtensionRegistration::new(key, move |world| {
                world
                    .register_component_type(component.clone())
                    .map_err(|error| {
                        WorldRuntimeExtensionError::registration_failed(&apply_key, error)
                    })
            }));
        }
        for (_, resource) in self.plugin_resources() {
            let resource = resource.clone();
            let key = format!("resource:{}", resource.type_name());
            registrations.push(WorldRuntimeExtensionRegistration::new(key, move |world| {
                resource.apply(world);
                Ok(())
            }));
        }
        for (_, event) in self.plugin_events() {
            let event = event.clone();
            let key = format!("event:{}", event.type_name());
            let apply_key = key.clone();
            registrations.push(WorldRuntimeExtensionRegistration::new(key, move |world| {
                event.apply(world).map_err(|error| {
                    WorldRuntimeExtensionError::registration_failed(&apply_key, error)
                })
            }));
        }
        for (_, system) in self.plugin_systems() {
            let registration = system.clone();
            let key = format!("system:{}", registration.id);
            let apply_key = key.clone();
            registrations.push(WorldRuntimeExtensionRegistration::new(key, move |world| {
                let system = registration.build(world).map_err(|error| {
                    WorldRuntimeExtensionError::registration_failed(&apply_key, error)
                })?;
                world.register_boxed_native_system(system).map_err(|error| {
                    WorldRuntimeExtensionError::registration_failed(&apply_key, error)
                })
            }));
        }
        for (_, system) in self.plugin_runtime_systems() {
            let registration = system.clone();
            let key = format!("system:{}", registration.id);
            let apply_key = key.clone();
            registrations.push(WorldRuntimeExtensionRegistration::new(key, move |world| {
                world
                    .register_boxed_runtime_scene_system(registration.build())
                    .map_err(|error| {
                        WorldRuntimeExtensionError::registration_failed(&apply_key, error)
                    })
            }));
        }
        WorldRuntimeExtensionPlan::from_registrations(registrations)
            .map_err(|error| RuntimeExtensionRegistryError::WorldRegistration(error.to_string()))
    }
}
