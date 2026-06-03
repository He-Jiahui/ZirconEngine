use crate::asset::{
    AssetImporterDescriptor, AssetImporterHandler, DiagnosticOnlyAssetImporter,
    NativeAssetImporterHandler,
};
use crate::graphics::{
    HybridGiRuntimeProviderRegistration, RenderFeatureDescriptor, RenderPassExecutorRegistration,
    RuntimePrepareCollectorRegistration, SolariRuntimeProviderRegistration,
    VirtualGeometryRuntimeProviderRegistration,
};
use crate::plugin::{LoadedNativePlugin, RuntimeExtensionRegistryError};
use std::sync::Arc;

use super::RuntimeExtensionRegistry;

mod metadata;
mod runtime_core;
mod scene_hook;

impl RuntimeExtensionRegistry {
    pub fn register_render_feature(
        &mut self,
        descriptor: RenderFeatureDescriptor,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        if self
            .render_features
            .iter()
            .any(|existing| existing.name == descriptor.name)
        {
            return Err(RuntimeExtensionRegistryError::DuplicateRenderFeature(
                descriptor.name,
            ));
        }
        self.render_features.push(descriptor);
        Ok(())
    }

    pub fn register_render_pass_executor(
        &mut self,
        registration: RenderPassExecutorRegistration,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        if self
            .render_pass_executors
            .iter()
            .any(|existing| existing.executor_id() == registration.executor_id())
        {
            return Err(RuntimeExtensionRegistryError::DuplicateRenderPassExecutor(
                registration.executor_id().to_string(),
            ));
        }
        self.render_pass_executors.push(registration);
        Ok(())
    }

    pub fn register_runtime_prepare_collector(
        &mut self,
        registration: RuntimePrepareCollectorRegistration,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        if self
            .runtime_prepare_collectors
            .iter()
            .any(|existing| existing.collector_id() == registration.collector_id())
        {
            return Err(
                RuntimeExtensionRegistryError::DuplicateRuntimePrepareCollector(
                    registration.collector_id().to_string(),
                ),
            );
        }
        self.runtime_prepare_collectors.push(registration);
        Ok(())
    }

    pub fn register_virtual_geometry_runtime_provider(
        &mut self,
        registration: VirtualGeometryRuntimeProviderRegistration,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        if self
            .virtual_geometry_runtime_providers
            .iter()
            .any(|existing| existing.provider_id() == registration.provider_id())
        {
            return Err(
                RuntimeExtensionRegistryError::DuplicateVirtualGeometryRuntimeProvider(
                    registration.provider_id().to_string(),
                ),
            );
        }
        self.virtual_geometry_runtime_providers.push(registration);
        Ok(())
    }

    pub fn register_hybrid_gi_runtime_provider(
        &mut self,
        registration: HybridGiRuntimeProviderRegistration,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        if self
            .hybrid_gi_runtime_providers
            .iter()
            .any(|existing| existing.provider_id() == registration.provider_id())
        {
            return Err(
                RuntimeExtensionRegistryError::DuplicateHybridGiRuntimeProvider(
                    registration.provider_id().to_string(),
                ),
            );
        }
        self.hybrid_gi_runtime_providers.push(registration);
        Ok(())
    }

    pub fn register_solari_runtime_provider(
        &mut self,
        registration: SolariRuntimeProviderRegistration,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        if self
            .solari_runtime_providers
            .iter()
            .any(|existing| existing.provider_id() == registration.provider_id())
        {
            return Err(
                RuntimeExtensionRegistryError::DuplicateSolariRuntimeProvider(
                    registration.provider_id().to_string(),
                ),
            );
        }
        self.solari_runtime_providers.push(registration);
        Ok(())
    }

    pub fn register_asset_importer(
        &mut self,
        importer: impl AssetImporterHandler + 'static,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        self.asset_importers
            .register(importer)
            .map_err(|error| RuntimeExtensionRegistryError::AssetImporter(error.to_string()))
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
        self.asset_importers
            .register_arc(importer)
            .map_err(|error| RuntimeExtensionRegistryError::AssetImporter(error.to_string()))
    }
}
