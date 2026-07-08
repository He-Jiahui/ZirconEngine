use super::support::assert_contains_all;

pub(super) fn assert_sdk_builder_mirrors_capabilities() {
    let feature_bundle_builder = include_str!(
        "../../../../../../../zircon_plugins/plugin_sdk/src/manifest/feature_bundle_builder.rs"
    );
    let manifest_mod =
        include_str!("../../../../../../../zircon_plugins/plugin_sdk/src/manifest/mod.rs");
    let plugin_sdk_lib = include_str!("../../../../../../../zircon_plugins/plugin_sdk/src/lib.rs");
    let plugin_sdk_prelude =
        include_str!("../../../../../../../zircon_plugins/plugin_sdk/src/prelude.rs");
    let editor_sdk = include_str!("../../../../../../../zircon_plugins/plugin_sdk/src/editor.rs");
    let manifest_tests =
        include_str!("../../../../../../../zircon_plugins/plugin_sdk/src/manifest/tests.rs");

    assert_contains_all(
        "SDK feature bundle builder mirrors capability declarations into feature and module manifests",
        feature_bundle_builder,
        &[
            "pub struct PluginFeatureBundleBuilder",
            "pub fn with_runtime_capability_module",
            "pub fn with_editor_capability_module",
            "let capability = capability.into();",
            "PluginModuleManifest::runtime(module_name, crate_name)",
            ".with_target_modes(target_modes)",
            ".with_capabilities([capability.clone()])",
            "self.with_capability(capability).with_runtime_module(module)",
            "PluginModuleManifest::editor(module_name, crate_name)",
            "self.with_capability(capability).with_editor_module(module)",
        ],
    );
    assert_contains_all(
        "SDK builder and mirror APIs are exported through stable plugin SDK surfaces",
        &format!("{manifest_mod}\n{plugin_sdk_lib}\n{plugin_sdk_prelude}\n{editor_sdk}"),
        &[
            "pub use feature_bundle_builder::PluginFeatureBundleBuilder",
            "PluginFeatureBundleBuilder",
            "pub fn mirrors_runtime(",
            "pub fn mirrors_runtime_manifest(",
            "pub fn mirrored_runtime_package_id(",
            "mirrors_runtime: $runtime_declaration:expr",
            "editor_declaration_mirrors_runtime_manifest_and_keeps_editor_capabilities",
        ],
    );
    assert_contains_all(
        "SDK manifest tests lock the builder capability projection contract",
        manifest_tests,
        &[
            "feature_bundle_builder_projects_capability_to_feature_and_modules",
            "with_runtime_capability_module",
            "with_editor_capability_module",
            "feature.capabilities",
            "feature.modules[0].capabilities",
            "feature.modules[1].capabilities",
        ],
    );
}
