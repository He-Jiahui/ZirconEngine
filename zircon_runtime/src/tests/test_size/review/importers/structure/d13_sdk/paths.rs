//! 为插件导入接口审查集中声明路径、子模块与锚点清单；消费者把这些值用于源码检查，清单中的名称不证明对应行为已执行。
pub(super) const PLUGIN_IMPORTER_D13_SOURCE_PATH: &str =
    "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d13_importer_sdk.rs";
pub(super) const PLUGIN_IMPORTER_D13_MANIFEST_PARITY_SOURCE_PATH: &str = "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d13_importer_sdk/manifest_parity.rs";
pub(super) const PLUGIN_IMPORTER_D13_RUNTIME_CRATES_SOURCE_PATH: &str = "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d13_importer_sdk/runtime_crates.rs";
pub(super) const PLUGIN_IMPORTER_D13_RUNTIME_EXPORTS_SOURCE_PATH: &str = "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d13_importer_sdk/runtime_exports.rs";
pub(super) const PLUGIN_IMPORTER_D13_RUNTIME_MANIFESTS_SOURCE_PATH: &str = "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d13_importer_sdk/runtime_manifests.rs";
