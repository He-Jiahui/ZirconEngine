use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::plugin::PluginModuleKind;

use crate::{
    neural_post_process_feature_manifest, package_manifest, runtime_plugin_descriptor,
    NATIVE_RUNTIME_ENTRY, NEURAL_POST_PROCESS_FEATURE_ID, NEURAL_POST_PROCESS_RUNTIME_CAPABILITY,
    PLUGIN_ID, RENDERING_POST_PROCESS_RUNTIME_CAPABILITY,
};

#[test]
fn neural_runtime_descriptor_uses_the_external_plugin_key_contract() {
    assert_eq!(runtime_plugin_descriptor().runtime_id().key(), PLUGIN_ID);
}

#[test]
fn neural_runtime_manifest_owns_the_post_process_feature_contract() {
    let package = package_manifest();
    let distribution = package
        .distribution
        .as_ref()
        .expect("neural runtime package must declare its native distribution");
    assert_eq!(distribution.runtime_entry, NATIVE_RUNTIME_ENTRY.name());
    assert_eq!(
        distribution.descriptor_symbol,
        zircon_plugin_sdk::NATIVE_DESCRIPTOR_SYMBOL_V3
    );
    assert_eq!(
        distribution.abi_version,
        Some(zircon_plugin_sdk::NATIVE_ABI_VERSION_V3)
    );
    assert_eq!(package.optional_features.len(), 1);

    let feature = &package.optional_features[0];
    assert_eq!(feature, &neural_post_process_feature_manifest());
    assert_eq!(feature.id, NEURAL_POST_PROCESS_FEATURE_ID);
    assert_eq!(feature.owner_plugin_id, "neural");
    assert!(!feature.enabled_by_default);
    assert_eq!(feature.dependencies.len(), 2);
    assert!(feature.dependencies.iter().any(|dependency| {
        dependency.plugin_id == "neural"
            && dependency.capability == "runtime.plugin.neural"
            && dependency.primary
    }));
    assert!(feature.dependencies.iter().any(|dependency| {
        dependency.plugin_id == "rendering"
            && dependency.capability == RENDERING_POST_PROCESS_RUNTIME_CAPABILITY
            && !dependency.primary
    }));
    assert_eq!(
        feature.capabilities,
        [NEURAL_POST_PROCESS_RUNTIME_CAPABILITY]
    );

    let module = feature
        .modules
        .iter()
        .find(|module| module.kind == PluginModuleKind::Runtime)
        .expect("neural post-process feature must expose one runtime module");
    assert_eq!(module.name, "neural.post_process.runtime");
    assert_eq!(
        module.crate_name,
        "zircon_plugin_neural_post_process_runtime"
    );
    assert_eq!(
        module.target_modes,
        [
            RuntimeTargetMode::ClientRuntime,
            RuntimeTargetMode::EditorHost,
        ]
    );
    assert_eq!(
        module.capabilities,
        [NEURAL_POST_PROCESS_RUNTIME_CAPABILITY]
    );

    let linked_registration =
        zircon_plugin_neural_post_process_runtime::plugin_feature_registration();
    assert!(
        linked_registration.is_success(),
        "{:?}",
        linked_registration.diagnostics
    );
    assert_eq!(linked_registration.manifest, feature.clone());
}
