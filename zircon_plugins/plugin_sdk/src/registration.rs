use std::sync::Arc;

use zircon_runtime::core::CoreError;
use zircon_runtime::core::framework::bridge::PluginInterface;
use zircon_runtime::core::framework::scene::ComponentTypeDescriptor;
use zircon_runtime::plugin::{
    BridgeImport, PluginEventCatalogManifest, PluginEventManifest, PluginModuleId,
    PluginOptionManifest, RuntimeExtensionRegistry, RuntimeExtensionRegistryError,
};
use zircon_runtime::scene::ecs::{
    Event, Resource, RuntimeSceneSystemContext, SceneSystemClockDomain, SystemOrderingConstraint,
    SystemRef, SystemStage,
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
    ) -> Result<RuntimePluginModuleRegistration<'registry>, RuntimeExtensionRegistryError> {
        let module_name = module_name.into();
        let owner = self.registry.intern_plugin_module(module_name.clone())?;
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

    pub fn owner(&self) -> PluginModuleId {
        self.owner
    }

    /// Registers a factory that produces a fresh callback for every runtime scene-system instance.
    pub fn runtime_scene_system<S>(
        &mut self,
        id: impl Into<String>,
        stage: SystemStage,
        system_factory: impl Fn() -> S + Send + Sync + 'static,
    ) -> RuntimePluginRuntimeSceneSystemBuilder<'_, S>
    where
        S: FnMut(RuntimeSceneSystemContext<'_>) -> Result<(), CoreError> + Send + 'static,
    {
        RuntimePluginRuntimeSceneSystemBuilder {
            registry: self.registry,
            owner: self.owner,
            id: id.into(),
            stage,
            system_factory: Arc::new(system_factory),
            sets: Vec::new(),
            constraints: Vec::new(),
            order: 0,
            clock_domain: SceneSystemClockDomain::Virtual,
        }
    }

    pub fn resource<T>(
        &mut self,
        init: impl FnMut() -> T + Send + 'static,
    ) -> Result<(), RuntimeExtensionRegistryError>
    where
        T: Resource,
    {
        self.registry.register_resource::<T>(self.owner, init)
    }

    pub fn component(
        &mut self,
        descriptor: ComponentTypeDescriptor,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        self.registry
            .register_component_for_owner(self.owner, descriptor)
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

    pub fn import_interface<T>(&mut self) -> Result<BridgeImport<T>, RuntimeExtensionRegistryError>
    where
        T: PluginInterface + ?Sized,
    {
        self.registry.import_interface::<T>(self.owner)
    }

    pub fn owner_revocation_listener(
        &mut self,
        callback: impl Fn(PluginModuleId) + Send + Sync + 'static,
    ) {
        self.registry
            .register_owner_revocation_listener(self.owner, callback);
    }
}

pub struct RuntimePluginRuntimeSceneSystemBuilder<'registry, S> {
    registry: &'registry mut RuntimeExtensionRegistry,
    owner: PluginModuleId,
    id: String,
    stage: SystemStage,
    system_factory: Arc<dyn Fn() -> S + Send + Sync>,
    sets: Vec<String>,
    constraints: Vec<SystemOrderingConstraint>,
    order: i32,
    clock_domain: SceneSystemClockDomain,
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

    pub fn with_clock_domain(mut self, clock_domain: SceneSystemClockDomain) -> Self {
        self.clock_domain = clock_domain;
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
        let system_factory = self.system_factory;

        let mut builder = self
            .registry
            .register_runtime_scene_system(self.owner, self.id, self.stage, move || {
                system_factory()
            })
            .with_order(self.order)
            .with_clock_domain(self.clock_domain);

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
    const DEFAULT_DOMAIN_SYSTEM_ID: &str = "sdk_registration.runtime.virtual-default";
    const OPTION_ID: &str = "sdk_registration.option";
    const CATALOG_NAMESPACE: &str = "sdk_registration.events";
    const CATALOG_SEED_EVENT_ID: &str = "sdk_registration.events.seed";
    const CATALOG_SEED_EVENT_SCHEMA: &str = "sdk_registration.seed.v1";
    const EVENT_ID: &str = "sdk_registration.events.runtime_event";
    const EVENT_SCHEMA: &str = "sdk_registration.runtime_event.v1";
    const WORLD_TRANSFORM_SYSTEM: &str = "zircon.scene.world_transform";

    #[derive(Clone, Debug)]
    struct SdkRegistrationEvent;

    #[derive(Clone, Debug, Default)]
    struct SdkRegistrationResource;

    impl Resource for SdkRegistrationResource {}

    trait SdkImportedBridge: Send + Sync {}

    impl PluginInterface for dyn SdkImportedBridge {
        const INTERFACE_ID: &'static str = "sdk.registration.imported.v1";
    }

    #[test]
    fn runtime_registration_builder_hides_module_owner_sequence() {
        let mut registry = RuntimeExtensionRegistry::default();
        registry
            .register_module(zircon_runtime::core::ModuleDescriptor::new(
                MODULE_NAME,
                "SDK registration builder test module",
            ))
            .expect("descriptor registered by runtime plugin report");
        let mut module = RuntimePluginRegistrationBuilder::new(&mut registry)
            .module(MODULE_OWNER)
            .expect("module registered");

        assert_eq!(module.module_name(), MODULE_OWNER);
        let module_owner = module.owner();
        let imported = module
            .import_interface::<dyn SdkImportedBridge>()
            .expect("interface import registered through SDK");

        module
            .component(ComponentTypeDescriptor::new(
                "sdk_registration.weather",
                "sdk_registration",
                "SDK Weather",
            ))
            .expect("component registered through SDK");

        module
            .resource(SdkRegistrationResource::default)
            .expect("runtime resource registered");
        module
            .runtime_scene_system(SYSTEM_ID, SystemStage::PostUpdate, || {
                |_context| Ok::<_, CoreError>(())
            })
            .in_set(SYSTEM_SET)
            .after(SystemRef::System(WORLD_TRANSFORM_SYSTEM.to_string()))
            .with_order(7)
            .with_clock_domain(SceneSystemClockDomain::Real)
            .register()
            .expect("runtime scene system registered");
        module
            .runtime_scene_system(DEFAULT_DOMAIN_SYSTEM_ID, SystemStage::Update, || {
                |_context| Ok::<_, CoreError>(())
            })
            .register()
            .expect("default-domain runtime scene system registered");
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
        drop(module);
        registry.finalize();
        assert_eq!(
            imported.call(|_| ()),
            Err(zircon_runtime::core::framework::bridge::BridgeError::Absent)
        );
        assert!(
            registry
                .components()
                .iter()
                .any(|component| component.type_id == "sdk_registration.weather")
        );

        assert!(
            registry
                .modules()
                .iter()
                .any(|module| module.name == MODULE_NAME)
        );

        let systems = registry.plugin_runtime_systems().collect::<Vec<_>>();
        assert_eq!(systems.len(), 2);
        let (owner, system) = systems
            .iter()
            .copied()
            .find(|(_, system)| system.id == SYSTEM_ID)
            .expect("explicit real-domain runtime system registered");
        assert_eq!(owner, module_owner);
        assert_eq!(registry.plugin_module_name(owner), Some(MODULE_OWNER));
        assert_eq!(system.id, SYSTEM_ID);
        assert_eq!(system.stage, SystemStage::PostUpdate);
        assert_eq!(system.order, 7);
        assert_eq!(system.clock_domain, SceneSystemClockDomain::Real);
        assert_eq!(system.sets.len(), 1);
        assert_eq!(
            system.constraints,
            vec![SystemOrderingConstraint::After(SystemRef::System(
                WORLD_TRANSFORM_SYSTEM.to_string()
            ))]
        );
        let (_, default_domain_system) = systems
            .iter()
            .copied()
            .find(|(_, system)| system.id == DEFAULT_DOMAIN_SYSTEM_ID)
            .expect("default-domain runtime system registered");
        assert_eq!(
            default_domain_system.clock_domain,
            SceneSystemClockDomain::Virtual
        );

        let events = registry.plugin_events().collect::<Vec<_>>();
        assert_eq!(events.len(), 1);
        let (event_owner, event) = events[0];
        assert_eq!(registry.plugin_module_name(event_owner), Some(MODULE_OWNER));
        assert_eq!(event.manifest().id, EVENT_ID);

        let resources = registry.plugin_resources().collect::<Vec<_>>();
        assert_eq!(resources.len(), 1);
        assert_eq!(
            registry.plugin_module_name(resources[0].0),
            Some(MODULE_OWNER)
        );
        assert_eq!(
            resources[0].1.type_name(),
            std::any::type_name::<SdkRegistrationResource>()
        );

        assert!(
            registry
                .plugin_options()
                .iter()
                .any(|option| option.key == OPTION_ID)
        );
        let event_catalog = registry
            .plugin_event_catalogs()
            .iter()
            .find(|catalog| catalog.namespace == CATALOG_NAMESPACE)
            .expect("event catalog registered");
        assert!(
            event_catalog
                .events
                .iter()
                .any(|event| event.id == EVENT_ID)
        );
    }

    #[test]
    fn runtime_registration_rejects_real_clock_domain_for_fixed_stages() {
        let mut registry = RuntimeExtensionRegistry::default();
        let mut module = RuntimePluginRegistrationBuilder::new(&mut registry)
            .module(MODULE_OWNER)
            .expect("module registered");

        let error = module
            .runtime_scene_system(
                "sdk_registration.runtime.invalid-fixed-real",
                SystemStage::FixedUpdate,
                || |_context| Ok::<_, CoreError>(()),
            )
            .with_clock_domain(SceneSystemClockDomain::Real)
            .register()
            .expect_err("fixed stages must reject the real-time clock domain");

        assert!(matches!(
            error,
            RuntimeExtensionRegistryError::InvalidPluginSystem(message)
                if message.contains("invalid-fixed-real")
                    && message.contains("FixedUpdate")
                    && message.contains("Real")
        ));
    }

    #[test]
    fn component_registration_rejects_foreign_owner_and_revokes_with_its_module() {
        let mut registry = RuntimeExtensionRegistry::default();
        let mut module = RuntimePluginRegistrationBuilder::new(&mut registry)
            .module("owner_a.runtime")
            .expect("module registered");

        let error = module
            .component(ComponentTypeDescriptor::new(
                "owner_b.Component.Foreign",
                "owner_b",
                "Foreign",
            ))
            .expect_err("builder must reject caller-forged component ownership");
        assert!(error.to_string().contains("owner_a"));
        assert!(error.to_string().contains("owner_b"));

        module
            .component(ComponentTypeDescriptor::new(
                "owner_a.Component.Local",
                "owner_a",
                "Local",
            ))
            .expect("matching owner component registered");
        let owner = module.owner();
        drop(module);

        assert_eq!(registry.components().len(), 1);
        let removed = registry.revoke_owner_registrations(owner);
        assert_eq!(removed.components.len(), 1);
        assert!(registry.components().is_empty());
    }
}
