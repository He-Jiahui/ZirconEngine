//! Editor-plugin descriptor and lifecycle contract.

use zircon_runtime::{
    core::framework::platform::RuntimeTargetMode, plugin::PluginEventConsumerManifest,
    plugin::PluginModuleManifest, plugin::PluginPackageManifest,
};

use crate::core::editor_extension::{EditorExtensionRegistry, EditorExtensionRegistryError};
use crate::core::runtime_event_consumer::EditorRuntimeEventConsumerRegistry;

use super::sdk::lifecycle::{EditorPluginLifecycleError, EditorPluginLifecycleEvent};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorPluginDescriptor {
    pub package_id: String,
    pub display_name: String,
    pub crate_name: String,
    pub category: String,
    pub capabilities: Vec<String>,
    pub event_consumers: Vec<PluginEventConsumerManifest>,
}

impl EditorPluginDescriptor {
    pub fn new(
        package_id: impl Into<String>,
        display_name: impl Into<String>,
        crate_name: impl Into<String>,
    ) -> Self {
        Self {
            package_id: package_id.into(),
            display_name: display_name.into(),
            crate_name: crate_name.into(),
            category: "uncategorized".to_string(),
            capabilities: Vec::new(),
            event_consumers: Vec::new(),
        }
    }

    pub fn with_category(mut self, category: impl Into<String>) -> Self {
        self.category = category.into();
        self
    }

    pub fn with_capability(mut self, capability: impl Into<String>) -> Self {
        self.capabilities.push(capability.into());
        self
    }

    pub fn with_event_consumer(mut self, consumer: PluginEventConsumerManifest) -> Self {
        self.event_consumers.push(consumer);
        self.event_consumers
            .sort_by(|left, right| left.consumer_id.cmp(&right.consumer_id));
        self
    }

    pub fn attach_to_package(&self, manifest: PluginPackageManifest) -> PluginPackageManifest {
        manifest.with_editor_module(
            PluginModuleManifest::editor(
                format!("{}.editor", self.package_id),
                self.crate_name.clone(),
            )
            .with_capabilities(self.capabilities.iter().cloned())
            .with_event_consumers(self.event_consumers.iter().cloned()),
        )
    }

    pub fn standalone_package_manifest(&self) -> PluginPackageManifest {
        PluginPackageManifest::new(self.package_id.clone(), self.display_name.clone())
            .with_category(self.category.clone())
            .with_supported_targets([RuntimeTargetMode::EditorHost])
            .with_capabilities(self.capabilities.iter().cloned())
    }

    pub fn builtin_catalog() -> Vec<Self> {
        super::catalog_gen::builtin_editor_plugin_descriptors()
    }
}

pub trait EditorPlugin {
    fn descriptor(&self) -> &EditorPluginDescriptor;

    fn package_manifest(&self, runtime_manifest: PluginPackageManifest) -> PluginPackageManifest {
        self.descriptor().attach_to_package(runtime_manifest)
    }

    fn editor_capabilities(&self) -> &[String] {
        &self.descriptor().capabilities
    }

    fn register_editor_extensions(
        &self,
        _registry: &mut EditorExtensionRegistry,
    ) -> Result<(), EditorExtensionRegistryError> {
        Ok(())
    }

    fn runtime_event_consumers(&self) -> EditorRuntimeEventConsumerRegistry {
        EditorRuntimeEventConsumerRegistry::default()
    }

    fn on_lifecycle_event(
        &self,
        _event: &EditorPluginLifecycleEvent,
    ) -> Result<(), EditorPluginLifecycleError> {
        Ok(())
    }
}

impl EditorPlugin for EditorPluginDescriptor {
    fn descriptor(&self) -> &EditorPluginDescriptor {
        self
    }
}
