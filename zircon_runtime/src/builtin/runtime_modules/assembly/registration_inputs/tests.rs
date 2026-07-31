use super::*;
use crate::builtin::RuntimePluginId;
use crate::core::framework::project::ProjectPluginSelection;
use crate::core::framework::render::{
    GBufferChannelMask, ShadingModelDescriptor, ShadingModelId, SHADING_MODEL_PLUGIN_ID_START,
};
use crate::plugin::{PluginPackageManifest, RuntimeExtensionRegistry};

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
    let registration = RuntimePluginRegistrationReport {
        package_manifest: PluginPackageManifest::new(plugin_key.clone(), "Toon Shading"),
        project_selection: ProjectPluginSelection::runtime_plugin(plugin_id, true, true),
        extensions,
        diagnostics: Vec::new(),
    };

    let inputs = registration_inputs_for_plugin_reports(&[&registration]);

    assert!(inputs.linked_plugin_ids().is_empty());
    assert_eq!(inputs.shading_models(), &[descriptor]);
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
