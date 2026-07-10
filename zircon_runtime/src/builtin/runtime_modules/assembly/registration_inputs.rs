use crate::asset::AssetImporterRegistry;
#[cfg(feature = "graphics")]
use crate::core::framework::render::{GeometrySourceDescriptor, ShadingModelDescriptor};
#[cfg(feature = "graphics")]
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
    #[cfg(feature = "graphics")]
    render_features: Vec<RenderFeatureDescriptor>,
    #[cfg(feature = "graphics")]
    geometry_sources: Vec<GeometrySourceDescriptor>,
    #[cfg(feature = "graphics")]
    shading_models: Vec<ShadingModelDescriptor>,
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

    pub(super) fn linked_plugin_ids(&self) -> &[String] {
        &self.linked_plugin_ids
    }

    pub(super) fn asset_importers(&self) -> &AssetImporterRegistry {
        &self.asset_importers
    }

    pub(super) fn asset_importer_errors(&self) -> &[String] {
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
            #[cfg(feature = "graphics")]
            render_features: extension_inputs.render_features,
            #[cfg(feature = "graphics")]
            geometry_sources: extension_inputs.geometry_sources,
            #[cfg(feature = "graphics")]
            shading_models: extension_inputs.shading_models,
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

#[cfg(all(test, feature = "graphics"))]
mod tests {
    use super::*;
    use crate::builtin::RuntimePluginId;
    use crate::core::framework::render::{
        GBufferChannelMask, ShadingModelDescriptor, ShadingModelId, SHADING_MODEL_PLUGIN_ID_START,
    };
    use crate::plugin::{PluginPackageManifest, ProjectPluginSelection, RuntimeExtensionRegistry};

    #[test]
    fn plugin_registration_inputs_collect_shading_model_descriptors() {
        let plugin_id = RuntimePluginId::new("toon_shading");
        let plugin_key = plugin_id.key().to_string();
        let descriptor = ShadingModelDescriptor::new(
            ShadingModelId::new(SHADING_MODEL_PLUGIN_ID_START),
            "custom:toon",
            "toon_forward",
            "toon_gbuffer",
            "toon_deferred",
            GBufferChannelMask::standard_lit(),
        );
        let mut extensions = RuntimeExtensionRegistry::default();
        extensions
            .register_shading_model(&plugin_key, descriptor.clone())
            .expect("plugin shading model descriptor registers");
        let registration = RuntimePluginRegistrationReport {
            package_manifest: PluginPackageManifest::new(plugin_key.clone(), "Toon Shading"),
            project_selection: ProjectPluginSelection::runtime_plugin(plugin_id, true, true),
            extensions,
            diagnostics: Vec::new(),
        };

        let inputs = registration_inputs_for_plugin_reports(&[&registration]);

        assert_eq!(
            inputs.linked_plugin_ids(),
            std::slice::from_ref(&plugin_key)
        );
        assert_eq!(inputs.shading_models(), &[descriptor]);
    }
}
