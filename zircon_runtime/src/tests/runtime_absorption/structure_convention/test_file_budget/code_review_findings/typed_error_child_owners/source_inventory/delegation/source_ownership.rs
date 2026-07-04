use super::super::super::super::super::*;
use super::super::*;

pub(super) fn assert_typed_error_source_inventory_paths_and_reads_are_child_owned(
    sources: &TypedErrorSourceInventorySources,
) {
    assert_contains_all(
        "typed-error source inventory paths child owns source paths",
        &sources.paths_child,
        &[
            "const TYPED_ERROR_SOURCE_PATHS",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/asset_loaders/texture.rs",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/live_host/replay_and_runtime/runtime_behavior.rs",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/shader_prewarm_cli/args_boundary.rs",
            "pub(super) fn typed_error_source_paths",
        ],
    );
    assert_contains_all(
        "typed-error source inventory reads child owns source aggregation helpers",
        &sources.reads_child,
        &[
            "pub(super) fn typed_error_sources",
            "pub(super) fn typed_error_children_source",
            "pub(super) fn typed_error_review_guard_count",
        ],
    );
    assert_eq!(
        typed_error_review_guard_count(),
        47,
        "typed-error source inventory should preserve all current F5/F6/F7 review guards"
    );
}
