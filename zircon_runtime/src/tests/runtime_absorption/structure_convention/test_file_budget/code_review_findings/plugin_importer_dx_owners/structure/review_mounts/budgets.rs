use super::*;

pub(super) fn assert_plugin_importer_dx_review_mounts_children_line_budgets_are_current(
    sources: &PluginImporterDxReviewMountSources,
) {
    for (path, source) in [
        (
            PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTIONS_CHILD,
            sources.structure_assertions_child.as_str(),
        ),
        (
            PLUGIN_IMPORTER_DX_REVIEW_MOUNTS_CHILD,
            sources.review_mounts_child.as_str(),
        ),
        (
            PLUGIN_IMPORTER_DX_REVIEW_MOUNTS_PATHS_CHILD,
            sources.paths_child.as_str(),
        ),
        (
            PLUGIN_IMPORTER_DX_REVIEW_MOUNTS_SOURCES_CHILD,
            sources.sources_child.as_str(),
        ),
        (
            PLUGIN_IMPORTER_DX_REVIEW_MOUNTS_PARENT_MOUNTS_CHILD,
            sources.parent_mounts_child.as_str(),
        ),
        (
            PLUGIN_IMPORTER_DX_REVIEW_MOUNTS_REVIEW_CHILDREN_CHILD,
            sources.review_children_child.as_str(),
        ),
        (
            PLUGIN_IMPORTER_DX_REVIEW_MOUNTS_BUDGETS_CHILD,
            sources.budgets_child.as_str(),
        ),
        (
            PLUGIN_IMPORTER_DX_REVIEW_MOUNTS_STATUS_MIRRORS_CHILD,
            sources.status_mirrors_child.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < PLUGIN_IMPORTER_DX_CHILD_OWNER_LINE_BUDGET,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
}

#[test]
fn runtime_15_plugin_importer_dx_review_mounts_children_line_budgets_are_current() {
    let sources = plugin_importer_dx_review_mount_sources();

    assert_plugin_importer_dx_review_mounts_children_line_budgets_are_current(&sources);
}
