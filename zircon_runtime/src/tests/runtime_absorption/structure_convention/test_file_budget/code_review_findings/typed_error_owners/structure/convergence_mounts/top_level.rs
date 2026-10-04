//! 约束类型化错误审查的模块挂载、委托入口与既有检查保留；读取当前源码后按文本验证，不能替代被检查模块的行为测试。
use super::super::super::super::super::*;
use super::*;

pub(super) fn assert_typed_error_convergence_top_level_parent_is_folder_backed(
    sources: &TypedErrorConvergenceMountSources,
) {
    assert_contains_all(
        "typed-error convergence parent mounts focused child owners",
        &sources.typed_error_parent,
        &[
            "mod animation_resource;",
            "mod asset_loaders;",
            "mod asset_records;",
            "mod diagnostics;",
            "mod dynamic_api;",
            "mod export_cli;",
            "mod native_plugin_loader;",
            "mod scene_world;",
            "mod script_host;",
            "mod shader_prewarm_cli;",
            "mod ui_asset_documents;",
            "mod ui_input;",
            "mod ui_template_resource;",
        ],
    );
    assert_eq!(
        sources.typed_error_parent.matches("#[test]").count(),
        0,
        "typed_error_convergence/mod.rs should only mount child test owners"
    );
}
