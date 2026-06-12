use crate::asset::AssetImporterRegistry;
use crate::graphics::{
    HybridGiRuntimeProviderRegistration, RenderFeatureDescriptor, RenderPassExecutorRegistration,
    RuntimePrepareCollectorRegistration, SolariRuntimeProviderRegistration,
    VirtualGeometryRuntimeProviderRegistration,
};
use crate::plugin::RuntimeExtensionRegistry;

pub(super) struct RuntimeModuleExtensionInputs {
    pub(super) asset_importers: AssetImporterRegistry,
    pub(super) asset_importer_errors: Vec<String>,
    pub(super) render_features: Vec<RenderFeatureDescriptor>,
    pub(super) render_pass_executors: Vec<RenderPassExecutorRegistration>,
    pub(super) runtime_prepare_collectors: Vec<RuntimePrepareCollectorRegistration>,
    pub(super) hybrid_gi_runtime_providers: Vec<HybridGiRuntimeProviderRegistration>,
    pub(super) solari_runtime_providers: Vec<SolariRuntimeProviderRegistration>,
    pub(super) virtual_geometry_runtime_providers: Vec<VirtualGeometryRuntimeProviderRegistration>,
}

pub(super) fn extension_inputs_from_extension_registries<'a>(
    registries: impl IntoIterator<Item = &'a RuntimeExtensionRegistry>,
) -> RuntimeModuleExtensionInputs {
    let registries = registries.into_iter().collect::<Vec<_>>();
    let (asset_importers, asset_importer_errors) =
        asset_importers_from_extension_registries(registries.iter().copied());
    RuntimeModuleExtensionInputs {
        asset_importers,
        asset_importer_errors,
        render_features: collect_render_features(&registries),
        render_pass_executors: collect_render_pass_executors(&registries),
        runtime_prepare_collectors: collect_runtime_prepare_collectors(&registries),
        hybrid_gi_runtime_providers: collect_hybrid_gi_runtime_providers(&registries),
        solari_runtime_providers: collect_solari_runtime_providers(&registries),
        virtual_geometry_runtime_providers: collect_virtual_geometry_runtime_providers(&registries),
    }
}

fn asset_importers_from_extension_registries<'a>(
    registries: impl IntoIterator<Item = &'a RuntimeExtensionRegistry>,
) -> (AssetImporterRegistry, Vec<String>) {
    let mut asset_importers = AssetImporterRegistry::default();
    let mut errors = Vec::new();
    for registry in registries {
        for importer in registry.asset_importers().importers() {
            if let Err(error) = asset_importers.register_arc(importer) {
                errors.push(format!("asset importer registration failed: {error}"));
            }
        }
    }
    (asset_importers, errors)
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
