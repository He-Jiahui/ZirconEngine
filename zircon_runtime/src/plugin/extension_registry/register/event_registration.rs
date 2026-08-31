use std::any::TypeId;
use std::fmt;

use crate::plugin::{
    PluginEventCatalogManifest, PluginEventManifest, RuntimeExtensionRegistryError,
};
use crate::scene::ecs::Event;
use crate::scene::{RuntimeEventMirrorError, RuntimeEventMirrorRegistration, SceneResult, World};
use serde::Serialize;

use super::super::RuntimeExtensionRegistry;
use super::super::owner::PluginModuleId;
use super::super::validation::validate_plugin_event_catalog_manifest;

#[derive(Clone)]
pub struct EventRegistration {
    type_id: TypeId,
    type_name: &'static str,
    manifest: PluginEventManifest,
    apply: EventApply,
}

#[derive(Clone)]
enum EventApply {
    Event(fn(&mut World)),
    Mirrored(RuntimeEventMirrorRegistration),
}

impl EventRegistration {
    fn new<E>(manifest: PluginEventManifest) -> Self
    where
        E: Event,
    {
        Self {
            type_id: TypeId::of::<E>(),
            type_name: std::any::type_name::<E>(),
            manifest,
            apply: EventApply::Event(|world| world.register_event::<E>()),
        }
    }

    fn mirrored<E>(
        manifest: PluginEventManifest,
        reader_count_callback: impl Fn(&mut World, u32) -> SceneResult<()> + Send + Sync + 'static,
    ) -> Self
    where
        E: Event + Serialize,
    {
        Self {
            type_id: TypeId::of::<E>(),
            type_name: std::any::type_name::<E>(),
            apply: EventApply::Mirrored(
                RuntimeEventMirrorRegistration::typed::<E>(
                    manifest.id.clone(),
                    manifest.payload_schema.clone(),
                )
                .with_reader_count_callback(reader_count_callback),
            ),
            manifest,
        }
    }

    pub fn type_name(&self) -> &'static str {
        self.type_name
    }

    pub fn manifest(&self) -> &PluginEventManifest {
        &self.manifest
    }

    pub(in crate::plugin::extension_registry) fn apply(
        &self,
        world: &mut World,
    ) -> Result<(), RuntimeEventMirrorError> {
        match &self.apply {
            EventApply::Event(apply) => {
                apply(world);
                Ok(())
            }
            EventApply::Mirrored(registration) => {
                world.register_runtime_event_mirror(registration.clone())
            }
        }
    }
}

impl fmt::Debug for EventRegistration {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("EventRegistration")
            .field("type_name", &self.type_name)
            .field("manifest", &self.manifest)
            .finish_non_exhaustive()
    }
}

impl RuntimeExtensionRegistry {
    pub fn register_event<E>(
        &mut self,
        owner: PluginModuleId,
        manifest: PluginEventManifest,
    ) -> Result<(), RuntimeExtensionRegistryError>
    where
        E: Event,
    {
        let namespace = self
            .plugin_modules
            .name(owner)
            .and_then(plugin_event_catalog_namespace_from_module)
            .ok_or_else(|| {
                RuntimeExtensionRegistryError::InvalidPluginModule(format!(
                    "unknown plugin module owner {}",
                    owner.raw()
                ))
            })?;
        validate_event_manifest(&namespace, &manifest)?;
        let registration = EventRegistration::new::<E>(manifest.clone());
        if self.plugin_events.contains_key(&registration.type_id) {
            return Err(RuntimeExtensionRegistryError::DuplicatePluginEvent(
                registration.type_name().to_string(),
            ));
        }
        self.push_derived_event_catalog_entry(namespace, manifest)?;
        self.register_event_registration(owner, registration)
    }

    pub fn register_mirrored_event<E>(
        &mut self,
        owner: PluginModuleId,
        manifest: PluginEventManifest,
        reader_count_callback: impl Fn(&mut World, u32) -> SceneResult<()> + Send + Sync + 'static,
    ) -> Result<(), RuntimeExtensionRegistryError>
    where
        E: Event + Serialize,
    {
        let namespace = self
            .plugin_modules
            .name(owner)
            .and_then(plugin_event_catalog_namespace_from_module)
            .ok_or_else(|| {
                RuntimeExtensionRegistryError::InvalidPluginModule(format!(
                    "unknown plugin module owner {}",
                    owner.raw()
                ))
            })?;
        validate_event_manifest(&namespace, &manifest)?;
        let registration =
            EventRegistration::mirrored::<E>(manifest.clone(), reader_count_callback);
        if self.plugin_events.contains_key(&registration.type_id) {
            return Err(RuntimeExtensionRegistryError::DuplicatePluginEvent(
                registration.type_name().to_string(),
            ));
        }
        self.push_derived_event_catalog_entry(namespace, manifest)?;
        self.register_event_registration(owner, registration)
    }

    pub(crate) fn register_event_registration(
        &mut self,
        owner: PluginModuleId,
        registration: EventRegistration,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        if self.plugin_events.contains_key(&registration.type_id) {
            return Err(RuntimeExtensionRegistryError::DuplicatePluginEvent(
                registration.type_name().to_string(),
            ));
        }
        self.plugin_events
            .register(owner, registration.type_id, registration)
            .expect("plugin event duplicate was prechecked");
        Ok(())
    }

    pub fn plugin_events(&self) -> impl Iterator<Item = (PluginModuleId, &EventRegistration)> {
        self.plugin_events
            .iter()
            .map(|(owner, _key, registration)| (owner, registration))
    }

    fn push_derived_event_catalog_entry(
        &mut self,
        namespace: String,
        manifest: PluginEventManifest,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        for catalog in self.plugin_event_catalogs.values_mut() {
            if catalog.namespace == namespace {
                if catalog.events.iter().any(|event| event.id == manifest.id) {
                    return Err(RuntimeExtensionRegistryError::DuplicatePluginEvent(
                        manifest.id,
                    ));
                }
                catalog.events.push(manifest);
                return Ok(());
            }
        }

        let owner = self.intern_owner_from_namespaced_key(&namespace)?;
        self.plugin_event_catalogs
            .register(
                owner,
                namespace.clone(),
                PluginEventCatalogManifest {
                    namespace,
                    version: 1,
                    events: vec![manifest],
                },
            )
            .expect("derived event catalog duplicate was prechecked");
        Ok(())
    }
}

fn plugin_event_catalog_namespace_from_module(module_name: &str) -> Option<String> {
    let plugin_id = module_name.split('.').next()?;
    if plugin_id.is_empty() {
        return None;
    }
    let capacity = plugin_id.len() + ".events".len();
    let mut namespace = String::with_capacity(capacity);
    namespace.push_str(plugin_id);
    namespace.push_str(".events");
    Some(namespace)
}

fn validate_event_manifest(
    namespace: &str,
    manifest: &PluginEventManifest,
) -> Result<(), RuntimeExtensionRegistryError> {
    validate_plugin_event_catalog_manifest(&PluginEventCatalogManifest {
        namespace: namespace.to_string(),
        version: 1,
        events: vec![manifest.clone()],
    })
}

#[cfg(test)]
mod tests {
    use super::plugin_event_catalog_namespace_from_module;

    #[test]
    fn exact_event_catalog_namespace_preserves_module_identity() {
        assert_eq!(
            plugin_event_catalog_namespace_from_module("weather.runtime"),
            Some("weather.events".to_string())
        );
        assert_eq!(
            plugin_event_catalog_namespace_from_module("weather"),
            Some("weather.events".to_string())
        );
        assert_eq!(plugin_event_catalog_namespace_from_module(".runtime"), None);
    }
}
