use std::sync::Arc;

use zircon_runtime::core::framework::bridge::PluginInterface;
use zircon_runtime::core::{CoreError, ModuleDescriptor};
use zircon_runtime::plugin::{
    PluginEventCatalogManifest, PluginEventManifest, PluginModuleId, PluginOptionManifest,
    RuntimeExtensionRegistry, RuntimeExtensionRegistryError,
};
use zircon_runtime::scene::ecs::{
    Event, RuntimeSceneSystemContext, SystemOrderingConstraint, SystemRef, SystemStage,
};

pub struct RuntimePluginRegistrationBuilder<'registry> {
    registry: &'registry mut RuntimeExtensionRegistry,
}

impl<'registry> RuntimePluginRegistrationBuilder<'registry> {
    pub fn new(registry: &'registry mut RuntimeExtensionRegistry) -> Self {
        Self { registry }
    }

    pub fn module(
        self,
        module_name: impl Into<String>,
        descriptor: ModuleDescriptor,
    ) -> Result<RuntimePluginModuleRegistration<'registry>, RuntimeExtensionRegistryError> {
        let module_name = module_name.into();
        let owner = self.registry.intern_plugin_module(module_name.clone())?;
        self.registry.register_module(descriptor)?;
        Ok(RuntimePluginModuleRegistration {
            registry: self.registry,
            module_name,
            owner,
        })
    }
}

pub struct RuntimePluginModuleRegistration<'registry> {
    registry: &'registry mut RuntimeExtensionRegistry,
    module_name: String,
    owner: PluginModuleId,
}

impl<'registry> RuntimePluginModuleRegistration<'registry> {
    pub fn module_name(&self) -> &str {
        &self.module_name
    }

    pub fn runtime_scene_system<S>(
        &mut self,
        id: impl Into<String>,
        stage: SystemStage,
        system: S,
    ) -> RuntimePluginRuntimeSceneSystemBuilder<'_, S>
    where
        S: FnMut(RuntimeSceneSystemContext<'_>) -> Result<(), CoreError> + Send + 'static,
    {
        RuntimePluginRuntimeSceneSystemBuilder {
            registry: self.registry,
            owner: self.owner,
            id: id.into(),
            stage,
            system,
            sets: Vec::new(),
            constraints: Vec::new(),
            order: 0,
        }
    }

    pub fn event<E>(
        &mut self,
        manifest: PluginEventManifest,
    ) -> Result<(), RuntimeExtensionRegistryError>
    where
        E: Event,
    {
        self.registry.register_event::<E>(self.owner, manifest)
    }

    pub fn plugin_option(
        &mut self,
        manifest: PluginOptionManifest,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        self.registry.register_plugin_option(manifest)
    }

    pub fn plugin_event_catalog(
        &mut self,
        manifest: PluginEventCatalogManifest,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        self.registry.register_plugin_event_catalog(manifest)
    }

    pub fn export_interface<T>(
        &mut self,
        implementation: Arc<T>,
    ) -> Result<(), RuntimeExtensionRegistryError>
    where
        T: PluginInterface + ?Sized,
    {
        self.registry
            .export_interface::<T>(self.owner, implementation)
    }
}

pub struct RuntimePluginRuntimeSceneSystemBuilder<'registry, S> {
    registry: &'registry mut RuntimeExtensionRegistry,
    owner: PluginModuleId,
    id: String,
    stage: SystemStage,
    system: S,
    sets: Vec<String>,
    constraints: Vec<SystemOrderingConstraint>,
    order: i32,
}

impl<'registry, S> RuntimePluginRuntimeSceneSystemBuilder<'registry, S>
where
    S: FnMut(RuntimeSceneSystemContext<'_>) -> Result<(), CoreError> + Send + 'static,
{
    pub fn in_set(mut self, set: impl Into<String>) -> Self {
        self.sets.push(set.into());
        self
    }

    pub fn with_order(mut self, order: i32) -> Self {
        self.order = order;
        self
    }

    pub fn before(mut self, reference: SystemRef) -> Self {
        self.constraints
            .push(SystemOrderingConstraint::Before(reference));
        self
    }

    pub fn after(mut self, reference: SystemRef) -> Self {
        self.constraints
            .push(SystemOrderingConstraint::After(reference));
        self
    }

    pub fn register(self) -> Result<(), RuntimeExtensionRegistryError> {
        let set_ids = self
            .sets
            .into_iter()
            .map(|set| self.registry.intern_system_set(set))
            .collect::<Result<Vec<_>, _>>()?;

        let mut builder = self
            .registry
            .register_runtime_scene_system(self.owner, self.id, self.stage, self.system)
            .with_order(self.order);

        for set in set_ids {
            builder = builder.in_set(set);
        }
        for constraint in self.constraints {
            builder = match constraint {
                SystemOrderingConstraint::Before(reference) => builder.before(reference),
                SystemOrderingConstraint::After(reference) => builder.after(reference),
            };
        }
        builder.register()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MODULE_OWNER: &str = "sdk_registration.runtime";
    const MODULE_NAME: &str = "SdkRegistrationRuntimeModule";
    const SYSTEM_SET: &str = "sdk_registration.update";
    const SYSTEM_ID: &str = "sdk_registration.runtime.tick";
    const OPTION_ID: &str = "sdk_registration.option";
    const CATALOG_NAMESPACE: &str = "sdk_registration.events";
    const CATALOG_SEED_EVENT_ID: &str = "sdk_registration.events.seed";
    const CATALOG_SEED_EVENT_SCHEMA: &str = "sdk_registration.seed.v1";
    const EVENT_ID: &str = "sdk_registration.events.runtime_event";
    const EVENT_SCHEMA: &str = "sdk_registration.runtime_event.v1";
    const WORLD_TRANSFORM_SYSTEM: &str = "zircon.scene.world_transform";

    #[derive(Clone, Debug)]
    struct SdkRegistrationEvent;

    #[test]
    fn runtime_registration_builder_hides_module_owner_sequence() {
        let mut registry = RuntimeExtensionRegistry::default();
        let mut module = RuntimePluginRegistrationBuilder::new(&mut registry)
            .module(
                MODULE_OWNER,
                ModuleDescriptor::new(MODULE_NAME, "SDK registration builder test module"),
            )
            .expect("module registered");

        assert_eq!(module.module_name(), MODULE_OWNER);

        module
            .runtime_scene_system(SYSTEM_ID, SystemStage::PostUpdate, |_context| {
                Ok::<_, CoreError>(())
            })
            .in_set(SYSTEM_SET)
            .after(SystemRef::System(WORLD_TRANSFORM_SYSTEM.to_string()))
            .with_order(7)
            .register()
            .expect("runtime scene system registered");
        module
            .plugin_option(PluginOptionManifest::new(
                OPTION_ID,
                "SDK Registration Option",
                "bool",
                "false",
            ))
            .expect("plugin option registered");
        module
            .plugin_event_catalog(PluginEventCatalogManifest {
                namespace: CATALOG_NAMESPACE.to_string(),
                version: 1,
                events: vec![PluginEventManifest {
                    id: CATALOG_SEED_EVENT_ID.to_string(),
                    display_name: "SDK Registration Seed Event".to_string(),
                    payload_schema: CATALOG_SEED_EVENT_SCHEMA.to_string(),
                }],
            })
            .expect("plugin event catalog registered");
        module
            .event::<SdkRegistrationEvent>(PluginEventManifest {
                id: EVENT_ID.to_string(),
                display_name: "SDK Registration Event".to_string(),
                payload_schema: EVENT_SCHEMA.to_string(),
            })
            .expect("runtime event registered");

        assert!(registry
            .modules()
            .iter()
            .any(|module| module.name == MODULE_NAME));

        let systems = registry.plugin_runtime_systems().collect::<Vec<_>>();
        assert_eq!(systems.len(), 1);
        let (owner, system) = systems[0];
        assert_eq!(registry.plugin_module_name(owner), Some(MODULE_OWNER));
        assert_eq!(system.id, SYSTEM_ID);
        assert_eq!(system.stage, SystemStage::PostUpdate);
        assert_eq!(system.order, 7);
        assert_eq!(system.sets.len(), 1);
        assert_eq!(
            system.constraints,
            vec![SystemOrderingConstraint::After(SystemRef::System(
                WORLD_TRANSFORM_SYSTEM.to_string()
            ))]
        );

        let events = registry.plugin_events().collect::<Vec<_>>();
        assert_eq!(events.len(), 1);
        let (event_owner, event) = events[0];
        assert_eq!(registry.plugin_module_name(event_owner), Some(MODULE_OWNER));
        assert_eq!(event.manifest().id, EVENT_ID);

        assert!(registry
            .plugin_options()
            .iter()
            .any(|option| option.key == OPTION_ID));
        let event_catalog = registry
            .plugin_event_catalogs()
            .iter()
            .find(|catalog| catalog.namespace == CATALOG_NAMESPACE)
            .expect("event catalog registered");
        assert!(event_catalog
            .events
            .iter()
            .any(|event| event.id == EVENT_ID));
    }
}
