use crate::asset::AssetImporterRegistry;
use crate::graphics::{
    HybridGiRuntimeProviderRegistration, RenderFeatureDescriptor, RenderPassExecutorRegistration,
    RuntimePrepareCollectorRegistration, SolariRuntimeProviderRegistration,
    VirtualGeometryRuntimeProviderRegistration,
};
use crate::plugin::{RuntimePluginFeatureRegistrationReport, RuntimePluginRegistrationReport};

use super::extension_inputs::{
    extension_inputs_from_extension_registries, RuntimeModuleExtensionInputs,
};

pub(super) struct RuntimeModuleRegistrationInputs {
    linked_plugin_ids: Vec<String>,
    asset_importers: AssetImporterRegistry,
    asset_importer_errors: Vec<String>,
    render_features: Vec<RenderFeatureDescriptor>,
    render_pass_executors: Vec<RenderPassExecutorRegistration>,
    runtime_prepare_collectors: Vec<RuntimePrepareCollectorRegistration>,
    hybrid_gi_runtime_providers: Vec<HybridGiRuntimeProviderRegistration>,
    solari_runtime_providers: Vec<SolariRuntimeProviderRegistration>,
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
            render_features: Vec::new(),
            render_pass_executors: Vec::new(),
            runtime_prepare_collectors: Vec::new(),
            hybrid_gi_runtime_providers: Vec::new(),
            solari_runtime_providers: Vec::new(),
            virtual_geometry_runtime_providers: Vec::new(),
        }
    }

    pub(super) fn linked_plugin_ids(&self) -> &[String] {
        &self.linked_plugin_ids
    }

    pub(super) fn asset_importers(&self) -> &AssetImporterRegistry {
        &self.asset_importers
    }

    pub(super) fn asset_importer_errors(&self) -> &[String] {
        &self.asset_importer_errors
    }

    pub(super) fn render_features(&self) -> &[RenderFeatureDescriptor] {
        &self.render_features
    }

    pub(super) fn render_pass_executors(&self) -> &[RenderPassExecutorRegistration] {
        &self.render_pass_executors
    }

    pub(super) fn runtime_prepare_collectors(&self) -> &[RuntimePrepareCollectorRegistration] {
        &self.runtime_prepare_collectors
    }

    pub(super) fn hybrid_gi_runtime_providers(&self) -> &[HybridGiRuntimeProviderRegistration] {
        &self.hybrid_gi_runtime_providers
    }

    pub(super) fn solari_runtime_providers(&self) -> &[SolariRuntimeProviderRegistration] {
        &self.solari_runtime_providers
    }

    pub(super) fn virtual_geometry_runtime_providers(
        &self,
    ) -> &[VirtualGeometryRuntimeProviderRegistration] {
        &self.virtual_geometry_runtime_providers
    }

    fn from_linked_plugin_ids_and_extension_inputs(
        linked_plugin_ids: impl IntoIterator<Item = impl AsRef<str>>,
        extension_inputs: RuntimeModuleExtensionInputs,
    ) -> Self {
        Self {
            linked_plugin_ids: linked_plugin_ids
                .into_iter()
                .map(|id| id.as_ref().to_string())
                .collect(),
            asset_importers: extension_inputs.asset_importers,
            asset_importer_errors: extension_inputs.asset_importer_errors,
            render_features: extension_inputs.render_features,
            render_pass_executors: extension_inputs.render_pass_executors,
            runtime_prepare_collectors: extension_inputs.runtime_prepare_collectors,
            hybrid_gi_runtime_providers: extension_inputs.hybrid_gi_runtime_providers,
            solari_runtime_providers: extension_inputs.solari_runtime_providers,
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
    RuntimeModuleRegistrationInputs::from_linked_plugin_ids_and_extension_inputs(
        registrations
            .iter()
            .map(|registration| registration.package_manifest.id.as_str()),
        extension_inputs,
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
    RuntimeModuleRegistrationInputs::from_linked_plugin_ids_and_extension_inputs(
        registrations
            .iter()
            .map(|registration| registration.package_manifest.id.as_str()),
        extension_inputs,
    )
}
