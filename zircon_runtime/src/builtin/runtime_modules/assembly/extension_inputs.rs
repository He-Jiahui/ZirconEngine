use crate::asset::AssetImporterRegistry;
#[cfg(feature = "graphics")]
use crate::core::framework::render::{GeometrySourceDescriptor, ShadingModelDescriptor};
#[cfg(feature = "graphics")]
use crate::graphics::{
    HybridGiRuntimeProviderRegistration, RenderFeatureDescriptor, RenderPassExecutorRegistration,
    RuntimePrepareCollectorRegistration, SolariRuntimeProviderRegistration,
    VirtualGeometryRuntimeProviderRegistration,
};
use crate::plugin::RuntimeExtensionRegistry;

pub(super) struct RuntimeModuleExtensionInputs {
    pub(super) asset_importers: AssetImporterRegistry,
    pub(super) asset_importer_errors: Vec<String>,
    #[cfg(feature = "graphics")]
    pub(super) render_features: Vec<RenderFeatureDescriptor>,
    #[cfg(feature = "graphics")]
    pub(super) geometry_sources: Vec<GeometrySourceDescriptor>,
    #[cfg(feature = "graphics")]
    pub(super) shading_models: Vec<ShadingModelDescriptor>,
    #[cfg(feature = "graphics")]
    pub(super) render_pass_executors: Vec<RenderPassExecutorRegistration>,
    #[cfg(feature = "graphics")]
    pub(super) runtime_prepare_collectors: Vec<RuntimePrepareCollectorRegistration>,
    #[cfg(feature = "graphics")]
    pub(super) hybrid_gi_runtime_providers: Vec<HybridGiRuntimeProviderRegistration>,
    #[cfg(feature = "graphics")]
    pub(super) solari_runtime_providers: Vec<SolariRuntimeProviderRegistration>,
    #[cfg(feature = "graphics")]
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
        #[cfg(feature = "graphics")]
        render_features: collect_render_features(&registries),
        #[cfg(feature = "graphics")]
        geometry_sources: collect_geometry_sources(&registries),
        #[cfg(feature = "graphics")]
        shading_models: collect_shading_models(&registries),
        #[cfg(feature = "graphics")]
        render_pass_executors: collect_render_pass_executors(&registries),
        #[cfg(feature = "graphics")]
        runtime_prepare_collectors: collect_runtime_prepare_collectors(&registries),
        #[cfg(feature = "graphics")]
        hybrid_gi_runtime_providers: collect_hybrid_gi_runtime_providers(&registries),
        #[cfg(feature = "graphics")]
        solari_runtime_providers: collect_solari_runtime_providers(&registries),
        #[cfg(feature = "graphics")]
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

#[cfg(feature = "graphics")]
fn collect_render_features(
    registries: &[&RuntimeExtensionRegistry],
) -> Vec<RenderFeatureDescriptor> {
    registries
        .iter()
        .flat_map(|registry| registry.render_features().iter().cloned())
        .collect()
}

#[cfg(feature = "graphics")]
fn collect_geometry_sources(
    registries: &[&RuntimeExtensionRegistry],
) -> Vec<GeometrySourceDescriptor> {
    registries
        .iter()
        .flat_map(|registry| registry.geometry_sources().iter().cloned())
        .collect()
}

#[cfg(feature = "graphics")]
fn collect_shading_models(registries: &[&RuntimeExtensionRegistry]) -> Vec<ShadingModelDescriptor> {
    registries
        .iter()
        .flat_map(|registry| registry.shading_models().iter().cloned())
        .collect()
}

#[cfg(feature = "graphics")]
fn collect_render_pass_executors(
    registries: &[&RuntimeExtensionRegistry],
) -> Vec<RenderPassExecutorRegistration> {
    registries
        .iter()
        .flat_map(|registry| registry.render_pass_executors().iter().cloned())
        .collect()
}

#[cfg(feature = "graphics")]
fn collect_runtime_prepare_collectors(
    registries: &[&RuntimeExtensionRegistry],
) -> Vec<RuntimePrepareCollectorRegistration> {
    registries
        .iter()
        .flat_map(|registry| registry.runtime_prepare_collectors().iter().cloned())
        .collect()
}

#[cfg(feature = "graphics")]
fn collect_hybrid_gi_runtime_providers(
    registries: &[&RuntimeExtensionRegistry],
) -> Vec<HybridGiRuntimeProviderRegistration> {
    registries
        .iter()
        .flat_map(|registry| registry.hybrid_gi_runtime_providers().iter().cloned())
        .collect()
}

#[cfg(feature = "graphics")]
fn collect_solari_runtime_providers(
    registries: &[&RuntimeExtensionRegistry],
) -> Vec<SolariRuntimeProviderRegistration> {
    registries
        .iter()
        .flat_map(|registry| registry.solari_runtime_providers().iter().cloned())
        .collect()
}

#[cfg(feature = "graphics")]
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
