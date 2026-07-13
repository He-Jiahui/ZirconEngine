use super::*;
use crate::builtin::RuntimePluginId;
use crate::core::framework::project::ProjectPluginSelection;
use crate::core::framework::render::{
    GBufferChannelMask, ShadingModelDescriptor, ShadingModelId, SHADING_MODEL_PLUGIN_ID_START,
};
use crate::plugin::{PluginPackageManifest, RuntimeExtensionRegistry};

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
