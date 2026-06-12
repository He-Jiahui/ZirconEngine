use std::any::TypeId;
use std::fmt;
use std::sync::{Arc, Mutex};

use crate::plugin::RuntimeExtensionRegistryError;
use crate::scene::ecs::Resource;
use crate::scene::World;

use super::super::owner::PluginModuleId;
use super::super::typed_extension_point::ExtensionSlot;
use super::super::RuntimeExtensionRegistry;

type ResourceApplyFn = Arc<dyn Fn(&mut World) + Send + Sync>;

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

    fn apply(&self, world: &mut World) {
        (self.inner)(world);
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
    fn new<T>(init: impl FnMut() -> T + Send + 'static) -> Self
    where
        T: Resource,
    {
        let type_name = std::any::type_name::<T>();
        let init = Arc::new(Mutex::new(init));
        Self {
            type_id: TypeId::of::<T>(),
            type_name,
            apply: SharedResourceApply::new(
                type_name,
                Arc::new(move |world| {
                    let mut init = init
                        .lock()
                        .expect("plugin resource initializer lock was poisoned");
                    world.insert_resource((*init)());
                }),
            ),
        }
    }

    pub fn type_name(&self) -> &'static str {
        self.type_name
    }

    pub(in crate::plugin::extension_registry) fn apply(&self, world: &mut World) {
        self.apply.apply(world);
    }
}

impl RuntimeExtensionRegistry {
    pub fn register_resource<T>(
        &mut self,
        owner: PluginModuleId,
        init: impl FnMut() -> T + Send + 'static,
    ) -> Result<(), RuntimeExtensionRegistryError>
    where
        T: Resource,
    {
        let registration = ResourceRegistration::new(init);
        self.register_resource_registration(owner, registration)
    }

    pub(crate) fn register_resource_registration(
        &mut self,
        owner: PluginModuleId,
        registration: ResourceRegistration,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        if self.plugin_resources.contains_key(&registration.type_id) {
            return Err(RuntimeExtensionRegistryError::DuplicatePluginResource(
                registration.type_name().to_string(),
            ));
        }
        self.plugin_resources
            .register(owner, registration.type_id, registration)
            .expect("plugin resource duplicate was prechecked");
        Ok(())
    }

    pub fn plugin_resources(
        &self,
    ) -> impl Iterator<Item = (PluginModuleId, &ResourceRegistration)> {
        self.plugin_resources
            .values()
            .iter()
            .enumerate()
            .filter_map(|(index, registration)| {
                let slot = ExtensionSlot::from_raw(index as u32);
                self.plugin_resources
                    .owner_for_slot(slot)
                    .map(|owner| (owner, registration))
            })
    }
}
