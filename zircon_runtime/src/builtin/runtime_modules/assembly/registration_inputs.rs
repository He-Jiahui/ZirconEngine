use crate::asset::AssetImporterRegistry;
use crate::graphics::{
    HybridGiRuntimeProviderRegistration, RenderFeatureDescriptor, RenderPassExecutorRegistration,
    RuntimePrepareCollectorRegistration, SolariRuntimeProviderRegistration,
    VirtualGeometryRuntimeProviderRegistration,
};
use crate::plugin::{
    RuntimeExtensionRegistry, RuntimePluginFeatureDependencyReport,
    RuntimePluginFeatureRegistrationReport, RuntimePluginRegistrationReport,
};

use super::super::extensions::asset_importers_from_extension_registries;
use super::super::RuntimeTargetMode;

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

    fn from_extensions_and_linked_plugins<'a>(
        linked_plugin_ids: impl IntoIterator<Item = impl AsRef<str>>,
        registries: impl IntoIterator<Item = &'a RuntimeExtensionRegistry>,
    ) -> Self {
        let registries = registries.into_iter().collect::<Vec<_>>();
        let (asset_importers, asset_importer_errors) =
            asset_importers_from_extension_registries(registries.iter().copied());
        Self {
            linked_plugin_ids: linked_plugin_ids
                .into_iter()
                .map(|id| id.as_ref().to_string())
                .collect(),
            asset_importers,
            asset_importer_errors,
            render_features: collect_render_features(&registries),
            render_pass_executors: collect_render_pass_executors(&registries),
            runtime_prepare_collectors: collect_runtime_prepare_collectors(&registries),
            hybrid_gi_runtime_providers: collect_hybrid_gi_runtime_providers(&registries),
            solari_runtime_providers: collect_solari_runtime_providers(&registries),
            virtual_geometry_runtime_providers: collect_virtual_geometry_runtime_providers(
                &registries,
            ),
        }
    }
}

pub(super) fn active_plugin_registration_refs<'a>(
    target: RuntimeTargetMode,
    registrations: impl IntoIterator<Item = &'a RuntimePluginRegistrationReport>,
) -> Vec<&'a RuntimePluginRegistrationReport> {
    registrations
        .into_iter()
        .filter(|registration| {
            registration.project_selection.enabled
                && registration.project_selection.supports_target(target)
        })
        .collect()
}

pub(super) fn active_feature_registration_refs<'a>(
    feature_registrations: &'a [RuntimePluginFeatureRegistrationReport],
    feature_report: &RuntimePluginFeatureDependencyReport,
) -> Vec<&'a RuntimePluginFeatureRegistrationReport> {
    feature_registrations
        .iter()
        .filter(|registration| {
            feature_report
                .available_features
                .iter()
                .any(|id| id == &registration.manifest.id)
        })
        .collect()
}

pub(super) fn registration_inputs_for_plugin_reports(
    registrations: &[&RuntimePluginRegistrationReport],
) -> RuntimeModuleRegistrationInputs {
    RuntimeModuleRegistrationInputs::from_extensions_and_linked_plugins(
        registrations
            .iter()
            .map(|registration| registration.package_manifest.id.as_str()),
        registrations
            .iter()
            .map(|registration| &registration.extensions),
    )
}

pub(super) fn registration_inputs_for_plugin_and_feature_reports(
    registrations: &[&RuntimePluginRegistrationReport],
    feature_registrations: &[&RuntimePluginFeatureRegistrationReport],
) -> RuntimeModuleRegistrationInputs {
    RuntimeModuleRegistrationInputs::from_extensions_and_linked_plugins(
        registrations
            .iter()
            .map(|registration| registration.package_manifest.id.as_str()),
        registrations
            .iter()
            .map(|registration| &registration.extensions)
            .chain(
                feature_registrations
                    .iter()
                    .map(|registration| &registration.extensions),
            ),
    )
}

fn collect_render_features(
    registries: &[&RuntimeExtensionRegistry],
) -> Vec<RenderFeatureDescriptor> {
    registries
        .iter()
        .flat_map(|registry| registry.render_features().iter().cloned())
        .collect()
}

fn collect_render_pass_executors(
    registries: &[&RuntimeExtensionRegistry],
) -> Vec<RenderPassExecutorRegistration> {
    registries
        .iter()
        .flat_map(|registry| registry.render_pass_executors().iter().cloned())
        .collect()
}

fn collect_runtime_prepare_collectors(
    registries: &[&RuntimeExtensionRegistry],
) -> Vec<RuntimePrepareCollectorRegistration> {
    registries
        .iter()
        .flat_map(|registry| registry.runtime_prepare_collectors().iter().cloned())
        .collect()
}

fn collect_hybrid_gi_runtime_providers(
    registries: &[&RuntimeExtensionRegistry],
) -> Vec<HybridGiRuntimeProviderRegistration> {
    registries
        .iter()
        .flat_map(|registry| registry.hybrid_gi_runtime_providers().iter().cloned())
        .collect()
}

fn collect_solari_runtime_providers(
    registries: &[&RuntimeExtensionRegistry],
) -> Vec<SolariRuntimeProviderRegistration> {
    registries
        .iter()
        .flat_map(|registry| registry.solari_runtime_providers().iter().cloned())
        .collect()
}

fn collect_virtual_geometry_runtime_providers(
    registries: &[&RuntimeExtensionRegistry],
) -> Vec<VirtualGeometryRuntimeProviderRegistration> {
    registries
        .iter()
        .flat_map(|registry| {
            registry
                .virtual_geometry_runtime_providers()
                .iter()
                .cloned()
        })
        .collect()
}
