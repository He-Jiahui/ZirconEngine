use crate::asset::{
    AssetImporterDescriptor, AssetImporterHandler, DiagnosticOnlyAssetImporter,
    NativeAssetImporterHandler,
};
#[cfg(feature = "graphics")]
use crate::graphics::{
    HybridGiRuntimeProviderRegistration, RenderFeatureDescriptor, RenderPassExecutorRegistration,
    RuntimePrepareCollectorRegistration, SolariRuntimeProviderRegistration,
    VirtualGeometryRuntimeProviderRegistration,
};
use crate::plugin::native::LoadedNativePlugin;
use crate::plugin::RuntimeExtensionRegistryError;
use crate::scene::ecs::SystemSetId;
use std::sync::Arc;

use super::owner::PluginModuleId;
use super::RuntimeExtensionRegistry;

mod bridge_registration;
mod event_registration;
mod metadata;
mod resource_registration;
mod runtime_core;
mod runtime_scene_system_registration;
mod scene_hook;
mod system_registration;

pub(in crate::plugin::extension_registry) use event_registration::EventRegistration;
pub(in crate::plugin::extension_registry) use resource_registration::ResourceRegistration;
pub(in crate::plugin::extension_registry) use runtime_scene_system_registration::RuntimeSceneSystemRegistration;
pub(in crate::plugin::extension_registry) use system_registration::SystemRegistration;

impl RuntimeExtensionRegistry {
    pub fn intern_plugin_module(
        &mut self,
        name: impl Into<String>,
    ) -> Result<PluginModuleId, RuntimeExtensionRegistryError> {
        self.plugin_modules.intern(name)
    }

    pub fn plugin_module_name(&self, owner: PluginModuleId) -> Option<&str> {
        self.plugin_modules.name(owner)
    }

    pub fn intern_system_set(
        &mut self,
        name: impl Into<String>,
    ) -> Result<SystemSetId, RuntimeExtensionRegistryError> {
        self.system_sets
            .intern(name)
            .map_err(|error| RuntimeExtensionRegistryError::InvalidPluginSystem(error.to_string()))
    }

    #[cfg(feature = "graphics")]
    pub fn register_render_feature(
        &mut self,
        descriptor: RenderFeatureDescriptor,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        if self.render_features.contains_key(&descriptor.name) {
            return Err(RuntimeExtensionRegistryError::DuplicateRenderFeature(
                descriptor.name,
            ));
        }
        let owner = self.intern_owner_from_namespaced_key(&descriptor.name)?;
        self.render_features
            .register(owner, descriptor.name.clone(), descriptor)
            .expect("render feature duplicate was prechecked");
        Ok(())
    }

    #[cfg(feature = "graphics")]
    pub fn register_render_pass_executor(
        &mut self,
        registration: RenderPassExecutorRegistration,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        let executor_id = registration.executor_id().to_string();
        if self.render_pass_executors.contains_key(&executor_id) {
            return Err(RuntimeExtensionRegistryError::DuplicateRenderPassExecutor(
                executor_id,
            ));
        }
        let owner = self.intern_owner_from_namespaced_key(&executor_id)?;
        self.render_pass_executors
            .register(owner, executor_id, registration)
            .expect("render pass executor duplicate was prechecked");
        Ok(())
    }

    #[cfg(feature = "graphics")]
    pub fn register_runtime_prepare_collector(
        &mut self,
        registration: RuntimePrepareCollectorRegistration,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        let collector_id = registration.collector_id().to_string();
        if self.runtime_prepare_collectors.contains_key(&collector_id) {
            return Err(
                RuntimeExtensionRegistryError::DuplicateRuntimePrepareCollector(collector_id),
            );
        }
        let owner = self.intern_owner_from_namespaced_key(&collector_id)?;
        self.runtime_prepare_collectors
            .register(owner, collector_id, registration)
            .expect("runtime prepare collector duplicate was prechecked");
        Ok(())
    }

    #[cfg(feature = "graphics")]
    pub fn register_virtual_geometry_runtime_provider(
        &mut self,
        registration: VirtualGeometryRuntimeProviderRegistration,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        let provider_id = registration.provider_id().to_string();
        if self
            .virtual_geometry_runtime_providers
            .contains_key(&provider_id)
        {
            return Err(
                RuntimeExtensionRegistryError::DuplicateVirtualGeometryRuntimeProvider(provider_id),
            );
        }
        let owner = self.intern_owner_from_namespaced_key(&provider_id)?;
        self.virtual_geometry_runtime_providers
            .register(owner, provider_id, registration)
            .expect("virtual geometry provider duplicate was prechecked");
        Ok(())
    }

    #[cfg(feature = "graphics")]
    pub fn register_hybrid_gi_runtime_provider(
        &mut self,
        registration: HybridGiRuntimeProviderRegistration,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        let provider_id = registration.provider_id().to_string();
        if self.hybrid_gi_runtime_providers.contains_key(&provider_id) {
            return Err(
                RuntimeExtensionRegistryError::DuplicateHybridGiRuntimeProvider(provider_id),
            );
        }
        let owner = self.intern_owner_from_namespaced_key(&provider_id)?;
        self.hybrid_gi_runtime_providers
            .register(owner, provider_id, registration)
            .expect("hybrid GI provider duplicate was prechecked");
        Ok(())
    }

    #[cfg(feature = "graphics")]
    pub fn register_solari_runtime_provider(
        &mut self,
        registration: SolariRuntimeProviderRegistration,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        let provider_id = registration.provider_id().to_string();
        if self.solari_runtime_providers.contains_key(&provider_id) {
            return Err(RuntimeExtensionRegistryError::DuplicateSolariRuntimeProvider(provider_id));
        }
        let owner = self.intern_owner_from_namespaced_key(&provider_id)?;
        self.solari_runtime_providers
            .register(owner, provider_id, registration)
            .expect("Solari provider duplicate was prechecked");
        Ok(())
    }

    pub fn register_asset_importer(
        &mut self,
        importer: impl AssetImporterHandler + 'static,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        let result = self
            .asset_importers
            .register(importer)
            .map_err(|error| RuntimeExtensionRegistryError::AssetImporter(error.to_string()));
        if result.is_ok() {
            self.asset_importers_finalized = false;
        }
        result
    }

    pub fn register_asset_importer_descriptor(
        &mut self,
        descriptor: AssetImporterDescriptor,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        let message = format!(
            "asset importer {} declared by plugin {} has no runtime backend attached",
            descriptor.id, descriptor.plugin_id
        );
        self.register_asset_importer(DiagnosticOnlyAssetImporter::new(descriptor, message))
    }

    pub fn register_native_asset_importer(
        &mut self,
        descriptor: AssetImporterDescriptor,
        plugin: Arc<LoadedNativePlugin>,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        self.register_asset_importer(NativeAssetImporterHandler::new(descriptor, plugin))
    }

    pub(crate) fn register_asset_importer_arc(
        &mut self,
        importer: Arc<dyn AssetImporterHandler>,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        let result = self
            .asset_importers
            .register_arc(importer)
            .map_err(|error| RuntimeExtensionRegistryError::AssetImporter(error.to_string()));
        if result.is_ok() {
            self.asset_importers_finalized = false;
        }
        result
    }

    pub(super) fn intern_runtime_owner(
        &mut self,
        plugin_id: &str,
    ) -> Result<PluginModuleId, RuntimeExtensionRegistryError> {
        self.intern_plugin_module(format!("{plugin_id}.runtime"))
    }

    pub(super) fn intern_owner_from_namespaced_key(
        &mut self,
        key: &str,
    ) -> Result<PluginModuleId, RuntimeExtensionRegistryError> {
        let Some(plugin_id) = key.split('.').next() else {
            return Err(RuntimeExtensionRegistryError::InvalidPluginModule(
                key.to_string(),
            ));
        };
        self.intern_runtime_owner(plugin_id)
    }
}
