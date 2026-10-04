use super::*;
use crate::builtin::RuntimePluginId;
use crate::core::framework::project::ProjectPluginSelection;
use crate::core::framework::render::{
    GBufferChannelMask, ShadingModelDescriptor, ShadingModelId, SHADING_MODEL_PLUGIN_ID_START,
};
use crate::plugin::{
    PluginFeatureBundleManifest, PluginPackageManifest, PluginShaderModuleSource,
    RuntimeExtensionRegistry, RuntimePluginFeatureRegistrationReport,
};

#[test]
fn plugin_registration_inputs_leave_provider_membership_to_availability_projection() {
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
    let shader_module_source = PluginShaderModuleSource::new(
        plugin_key.clone(),
        "zircon_toon::lighting",
        "fn zircon_toon_lighting() -> vec3f { return vec3f(0.5); }",
        "toon fixture shader module",
    );
    extensions
        .register_plugin_shader_module_source(&plugin_key, shader_module_source.clone())
        .expect("plugin shader module source registers");
    let registration = RuntimePluginRegistrationReport {
        package_manifest: PluginPackageManifest::new(plugin_key.clone(), "Toon Shading"),
        project_selection: ProjectPluginSelection::runtime_plugin(plugin_id, true, true),
        extensions,
        diagnostics: Vec::new(),
    };

    let inputs = registration_inputs_for_plugin_reports(&[&registration]);

    assert!(inputs.linked_plugin_ids().is_empty());
    assert_eq!(inputs.shading_models(), &[descriptor]);
    assert_eq!(
        inputs.plugin_shader_module_sources(),
        &[shader_module_source]
    );
}

#[test]
fn feature_registration_inputs_preserve_shader_modules_from_active_feature_extensions() {
    let package_id = "feature_extension_fixture";
    let source = PluginShaderModuleSource::new(
        package_id,
        "zircon_fixture::feature_extension",
        "fn feature_extension_lighting() -> vec3f { return vec3f(0.2); }",
        "feature extension fixture",
    );
    let mut feature_registration =
        RuntimePluginFeatureRegistrationReport::from_native_feature_manifest(
            PluginFeatureBundleManifest::new(
                "feature_extension_fixture.runtime",
                "Feature Extension Fixture",
                package_id,
            ),
            Some(package_id.to_string()),
        );
    feature_registration
        .extensions
        .register_plugin_shader_module_source(package_id, source.clone())
        .expect("feature extension shader module source registers");

    let inputs = registration_inputs_for_plugin_and_feature_reports(&[], &[&feature_registration]);

    assert_eq!(inputs.plugin_shader_module_sources(), &[source]);
}

#[test]
fn linked_plugin_inputs_build_final_membership_once() {
    let inputs = RuntimeModuleRegistrationInputs::from_linked_plugin_ids([
        "zircon.plugin.one",
        "zircon.plugin.one",
        "zircon.plugin.two",
    ]);

    assert_eq!(inputs.linked_plugin_ids().len(), 2);
    assert!(inputs.linked_plugin_ids().contains("zircon.plugin.one"));
    assert!(inputs.linked_plugin_ids().contains("zircon.plugin.two"));
}
