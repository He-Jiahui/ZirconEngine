use std::collections::HashSet;

use crate::asset::{AssetImporterRegistry, AssetImporterRegistryError};
#[cfg(feature = "graphics")]
use crate::core::framework::render::{GeometrySourceDescriptor, ShadingModelDescriptor};
#[cfg(feature = "graphics")]
use crate::graphics::{
    HybridGiRuntimeProviderRegistration, RenderFeatureDescriptor, RenderPassExecutorRegistration,
    RuntimePrepareCollectorRegistration, SolariRuntimeProviderRegistration,
    VirtualGeometryRuntimeProviderRegistration,
};
#[cfg(feature = "graphics")]
use crate::plugin::PluginShaderModuleSource;
use crate::plugin::{
    RuntimeExtensionRegistry, RuntimePluginFeatureRegistrationReport,
    RuntimePluginRegistrationReport,
};

use super::extension_inputs::{
    extension_inputs_from_extension_registries, RuntimeModuleExtensionInputs,
};

pub(super) struct RuntimeModuleRegistrationInputs {
    linked_plugin_ids: HashSet<String>,
    asset_importers: AssetImporterRegistry,
    asset_importer_errors: Vec<AssetImporterRegistryError>,
    #[cfg(feature = "graphics")]
    render_features: Vec<RenderFeatureDescriptor>,
    #[cfg(feature = "graphics")]
    geometry_sources: Vec<GeometrySourceDescriptor>,
    #[cfg(feature = "graphics")]
    shading_models: Vec<ShadingModelDescriptor>,
    #[cfg(feature = "graphics")]
    plugin_shader_module_sources: Vec<PluginShaderModuleSource>,
    #[cfg(feature = "graphics")]
    render_pass_executors: Vec<RenderPassExecutorRegistration>,
    #[cfg(feature = "graphics")]
    runtime_prepare_collectors: Vec<RuntimePrepareCollectorRegistration>,
    #[cfg(feature = "graphics")]
    hybrid_gi_runtime_providers: Vec<HybridGiRuntimeProviderRegistration>,
    #[cfg(feature = "graphics")]
    solari_runtime_providers: Vec<SolariRuntimeProviderRegistration>,
    #[cfg(feature = "graphics")]
    virtual_geometry_runtime_providers: Vec<VirtualGeometryRuntimeProviderRegistration>,
}

impl RuntimeModuleRegistrationInputs {
    pub(super) fn empty() -> Self {
        Self::from_linked_plugin_ids(std::iter::empty::<String>())
    }

    pub(super) fn from_linked_plugin_ids(
        linked_plugin_ids: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> Self {
        Self {
            linked_plugin_ids: linked_plugin_ids
                .into_iter()
                .map(|id| id.as_ref().to_string())
                .collect(),
            asset_importers: AssetImporterRegistry::default(),
            asset_importer_errors: Vec::new(),
            #[cfg(feature = "graphics")]
            render_features: Vec::new(),
            #[cfg(feature = "graphics")]
            geometry_sources: Vec::new(),
            #[cfg(feature = "graphics")]
            shading_models: Vec::new(),
            #[cfg(feature = "graphics")]
            plugin_shader_module_sources: Vec::new(),
            #[cfg(feature = "graphics")]
            render_pass_executors: Vec::new(),
            #[cfg(feature = "graphics")]
            runtime_prepare_collectors: Vec::new(),
            #[cfg(feature = "graphics")]
            hybrid_gi_runtime_providers: Vec::new(),
            #[cfg(feature = "graphics")]
            solari_runtime_providers: Vec::new(),
            #[cfg(feature = "graphics")]
            virtual_geometry_runtime_providers: Vec::new(),
        }
    }

    pub(super) fn linked_plugin_ids(&self) -> &HashSet<String> {
        &self.linked_plugin_ids
    }

    pub(super) fn asset_importers(&self) -> &AssetImporterRegistry {
        &self.asset_importers
    }

    pub(super) fn asset_importer_errors(&self) -> &[AssetImporterRegistryError] {
        &self.asset_importer_errors
    }

    #[cfg(feature = "graphics")]
    pub(super) fn render_features(&self) -> &[RenderFeatureDescriptor] {
        &self.render_features
    }

    #[cfg(feature = "graphics")]
    pub(super) fn geometry_sources(&self) -> &[GeometrySourceDescriptor] {
        &self.geometry_sources
    }

    #[cfg(feature = "graphics")]
    pub(super) fn shading_models(&self) -> &[ShadingModelDescriptor] {
        &self.shading_models
    }

    #[cfg(feature = "graphics")]
    pub(super) fn plugin_shader_module_sources(&self) -> &[PluginShaderModuleSource] {
        &self.plugin_shader_module_sources
    }

    #[cfg(feature = "graphics")]
    pub(super) fn render_pass_executors(&self) -> &[RenderPassExecutorRegistration] {
        &self.render_pass_executors
    }

    #[cfg(feature = "graphics")]
    pub(super) fn runtime_prepare_collectors(&self) -> &[RuntimePrepareCollectorRegistration] {
        &self.runtime_prepare_collectors
    }

    #[cfg(feature = "graphics")]
    pub(super) fn hybrid_gi_runtime_providers(&self) -> &[HybridGiRuntimeProviderRegistration] {
        &self.hybrid_gi_runtime_providers
    }

    #[cfg(feature = "graphics")]
    pub(super) fn solari_runtime_providers(&self) -> &[SolariRuntimeProviderRegistration] {
        &self.solari_runtime_providers
    }

    #[cfg(feature = "graphics")]
    pub(super) fn virtual_geometry_runtime_providers(
        &self,
    ) -> &[VirtualGeometryRuntimeProviderRegistration] {
        &self.virtual_geometry_runtime_providers
    }

    fn from_extension_inputs(extension_inputs: RuntimeModuleExtensionInputs) -> Self {
        Self {
            linked_plugin_ids: HashSet::new(),
            asset_importers: extension_inputs.asset_importers,
            asset_importer_errors: extension_inputs.asset_importer_errors,
            #[cfg(feature = "graphics")]
            render_features: extension_inputs.render_features,
            #[cfg(feature = "graphics")]
            geometry_sources: extension_inputs.geometry_sources,
            #[cfg(feature = "graphics")]
            shading_models: extension_inputs.shading_models,
            #[cfg(feature = "graphics")]
            plugin_shader_module_sources: extension_inputs.shader_module_sources,
            #[cfg(feature = "graphics")]
            render_pass_executors: extension_inputs.render_pass_executors,
            #[cfg(feature = "graphics")]
            runtime_prepare_collectors: extension_inputs.runtime_prepare_collectors,
            #[cfg(feature = "graphics")]
            hybrid_gi_runtime_providers: extension_inputs.hybrid_gi_runtime_providers,
            #[cfg(feature = "graphics")]
            solari_runtime_providers: extension_inputs.solari_runtime_providers,
            #[cfg(feature = "graphics")]
            virtual_geometry_runtime_providers: extension_inputs.virtual_geometry_runtime_providers,
        }
    }
}

pub(super) fn registration_inputs_for_plugin_reports(
    registrations: &[&RuntimePluginRegistrationReport],
) -> RuntimeModuleRegistrationInputs {
    let extension_inputs = extension_inputs_from_extension_registries(
        registrations
            .iter()
            .map(|registration| &registration.extensions),
    );
    RuntimeModuleRegistrationInputs::from_extension_inputs(extension_inputs)
}

pub(super) fn registration_inputs_for_extension_registry(
    registry: &RuntimeExtensionRegistry,
) -> RuntimeModuleRegistrationInputs {
    RuntimeModuleRegistrationInputs::from_extension_inputs(
        extension_inputs_from_extension_registries([registry]),
    )
}

pub(super) fn registration_inputs_for_plugin_and_feature_reports(
    registrations: &[&RuntimePluginRegistrationReport],
    feature_registrations: &[&RuntimePluginFeatureRegistrationReport],
) -> RuntimeModuleRegistrationInputs {
    let extension_inputs = extension_inputs_from_extension_registries(
        registrations
            .iter()
            .map(|registration| &registration.extensions)
            .chain(
                feature_registrations
                    .iter()
                    .map(|registration| &registration.extensions),
            ),
    );
    RuntimeModuleRegistrationInputs::from_extension_inputs(extension_inputs)
}

#[cfg(all(test, feature = "graphics"))]
mod tests;
