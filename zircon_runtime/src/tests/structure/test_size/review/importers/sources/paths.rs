//! 为插件导入接口审查读取或聚合父子源码，保留路径与文本的配对关系供上层检查；聚合结果只描述被列入清单的文件。
pub(super) const PLUGIN_IMPORTER_DX_SOURCE_PATHS: &[&str] = &[
    "tests/runtime_absorption/code_review_findings/plugin_importer_dx.rs",
    "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d10_bridge_call.rs",
    "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d1_capability_single_source.rs",
    "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d1_capability_single_source/audit_surfaces.rs",
    "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d1_capability_single_source/runtime_roots.rs",
    "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d1_capability_single_source/sdk_builder.rs",
    "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d1_capability_single_source/split_layout.rs",
    "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d1_capability_single_source/status_docs.rs",
    "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d1_capability_single_source/support.rs",
    "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d11_test_runtime_fixture.rs",
    "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d12_runtime_exports.rs",
    "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d13_importer_sdk.rs",
    "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d13_importer_sdk/manifest_parity.rs",
    "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d13_importer_sdk/runtime_crates.rs",
    "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d13_importer_sdk/runtime_exports.rs",
    "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d13_importer_sdk/runtime_manifests.rs",
    "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d5_editor_authoring.rs",
    "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d6_runtime_plugin_id.rs",
    "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d8_registration_builder.rs",
    "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d9_editor_runtime_mirror.rs",
];

pub(super) fn plugin_importer_dx_source_paths() -> &'static [&'static str] {
    PLUGIN_IMPORTER_DX_SOURCE_PATHS
}
