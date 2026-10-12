//! 约束插件导入接口审查的模块挂载、委托入口与既有检查保留；读取当前源码后按文本验证，不能替代被检查模块的行为测试。
use super::super::super::super::super::super::*;
use super::super::*;

pub(super) fn assert_plugin_importer_d13_sdk_parent_mounts_review_children(
    sources: &PluginImporterD13SdkStructureSources,
) {
    assert_contains_all(
        "plugin importer DX D13 parent mounts focused SDK review guard children",
        &sources.plugin_importer_dx_d13,
        &[
            "#[path = \"d13_importer_sdk/runtime_crates.rs\"]",
            "mod runtime_crates;",
            "#[path = \"d13_importer_sdk/runtime_exports.rs\"]",
            "mod runtime_exports;",
            "#[path = \"d13_importer_sdk/runtime_manifests.rs\"]",
            "mod runtime_manifests;",
            "#[path = \"d13_importer_sdk/manifest_parity.rs\"]",
            "mod manifest_parity;",
        ],
    );
    assert_eq!(
        sources.plugin_importer_dx_d13.matches("#[test]").count(),
        0,
        "plugin_importer_dx/d13_importer_sdk.rs should only mount child review guard owners"
    );
    for child_owned_test in [
        "fn review_d13_importer_runtime_exports_use_sdk_macro",
        "fn review_d13_importer_runtime_manifests_use_sdk_builder",
        "fn review_d13_importer_manifest_parity_guard_lives_in_sdk_builder",
    ] {
        assert!(
            !sources.plugin_importer_dx_d13.contains(child_owned_test),
            "child-owned D13 importer SDK review guard `{child_owned_test}` should not return to plugin_importer_dx/d13_importer_sdk.rs"
        );
    }
}
