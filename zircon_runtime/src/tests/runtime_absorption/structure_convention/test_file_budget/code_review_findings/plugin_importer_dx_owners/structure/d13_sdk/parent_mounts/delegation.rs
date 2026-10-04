//! 约束插件导入接口审查的模块挂载、委托入口与既有检查保留；读取当前源码后按文本验证，不能替代被检查模块的行为测试。
use super::super::super::super::super::super::*;
use super::super::*;

pub(super) fn assert_plugin_importer_d13_sdk_structure_parent_delegates(
    sources: &PluginImporterD13SdkStructureSources,
) {
    assert_contains_all(
        "plugin-importer DX structure assertions delegate D13 SDK structure checks to child owner",
        &sources.structure_assertions_child,
        &[
            "#[path = \"structure/d13_sdk.rs\"]",
            "mod d13_sdk;",
            "d13_sdk::assert_plugin_importer_d13_sdk_child_owners_are_folder_backed",
        ],
    );
}
