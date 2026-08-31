use std::any::TypeId;
use std::fmt;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Arc;

use crate::plugin::RuntimeExtensionRegistryError;
use crate::scene::ecs::Resource;
use crate::scene::World;

use super::super::owner::PluginModuleId;
use super::super::RuntimeExtensionRegistry;

#[cfg(test)]
#[path = "resource_registration/poison_recovery_tests.rs"]
mod poison_recovery_tests;

type ResourceApplyFn = Arc<dyn Fn(&mut World) -> Result<(), ResourceApplyError> + Send + Sync>;

#[derive(Debug)]
pub(in crate::plugin::extension_registry) struct ResourceApplyError(String);

impl ResourceApplyError {
    fn factory_panic(payload: Box<dyn std::any::Any + Send>) -> Self {
        Self(format!(
            "resource factory panicked: {}",
            panic_payload_message(payload)
        ))
    }
}

impl fmt::Display for ResourceApplyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Clone)]
struct SharedResourceApply {
    type_name: &'static str,
    inner: ResourceApplyFn,
}

impl SharedResourceApply {
    fn new(type_name: &'static str, apply: ResourceApplyFn) -> Self {
        Self {
            type_name,
            inner: apply,
        }
    }

    fn apply(&self, world: &mut World) -> Result<(), ResourceApplyError> {
        (self.inner)(world)
    }
}

impl fmt::Debug for SharedResourceApply {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SharedResourceApply")
            .field("type_name", &self.type_name)
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Debug)]
pub struct ResourceRegistration {
    type_id: TypeId,
    type_name: &'static str,
    apply: SharedResourceApply,
}

impl ResourceRegistration {
    fn new<T>(factory: impl Fn() -> T + Send + Sync + 'static) -> Self
    where
        T: Resource,
    {
        // A finalized plan can initialize multiple worlds, so its factory must
        // remain immutable and carry no shared per-world mutation state.
        let type_name = std::any::type_name::<T>();
        Self {
            type_id: TypeId::of::<T>(),
            type_name,
            apply: SharedResourceApply::new(
                type_name,
                Arc::new(move |world| {
                    let resource = catch_unwind(AssertUnwindSafe(|| factory()))
                        .map_err(ResourceApplyError::factory_panic)?;
                    world.insert_resource(resource);
                    Ok(())
                }),
            ),
        }
    }

    pub fn type_name(&self) -> &'static str {
        self.type_name
    }

    pub(in crate::plugin::extension_registry) fn apply(
        &self,
        world: &mut World,
    ) -> Result<(), ResourceApplyError> {
        self.apply.apply(world)
    }
}

fn panic_payload_message(payload: Box<dyn std::any::Any + Send>) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_owned()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "non-string panic payload".to_owned()
    }
}

impl RuntimeExtensionRegistry {
    /// Registers a repeatable factory that creates one fresh resource value for
    /// every world initialized from the extension plan.
    pub fn register_resource<T>(
        &mut self,
        owner: PluginModuleId,
        factory: impl Fn() -> T + Send + Sync + 'static,
    ) -> Result<(), RuntimeExtensionRegistryError>
    where
        T: Resource,
    {
        let registration = ResourceRegistration::new(factory);
        self.register_resource_registration(owner, registration)
    }

    pub(crate) fn register_resource_registration(
        &mut self,
        owner: PluginModuleId,
        registration: ResourceRegistration,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        let module_name = self.plugin_module_name(owner).ok_or_else(|| {
            RuntimeExtensionRegistryError::InvalidPluginModule(format!(
                "unknown plugin module owner {}",
                owner.raw()
            ))
        })?;
        if module_name.strip_suffix(".runtime").is_none() {
            return Err(RuntimeExtensionRegistryError::InvalidPluginModule(format!(
                "resource owner `{module_name}` must use the <plugin>.runtime module form"
            )));
        }
        let type_name = registration.type_name().to_string();
        self.plugin_resources
            .register(owner, registration.type_id, registration)
            .map_err(|_| RuntimeExtensionRegistryError::DuplicatePluginResource(type_name))?;
        Ok(())
    }

    pub fn plugin_resources(
        &self,
    ) -> impl Iterator<Item = (PluginModuleId, &ResourceRegistration)> {
        self.plugin_resources
            .iter()
            .map(|(owner, _key, registration)| (owner, registration))
    }
}
