use super::*;

pub(super) fn assert_plugin_importer_d13_sdk_structure_assertions_children_line_budgets_are_current(
    sources: &PluginImporterD13SdkStructureSources,
) {
    for (path, source) in [
        (
            PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTIONS_CHILD,
            sources.structure_assertions_child.as_str(),
        ),
        (
            PLUGIN_IMPORTER_D13_STRUCTURE_ASSERTIONS_CHILD,
            sources.d13_sdk_child.as_str(),
        ),
        (
            PLUGIN_IMPORTER_D13_PATHS_CHILD,
            sources.paths_child.as_str(),
        ),
        (
            PLUGIN_IMPORTER_D13_SOURCES_CHILD,
            sources.sources_child.as_str(),
        ),
        (
            PLUGIN_IMPORTER_D13_PARENT_MOUNTS_CHILD,
            sources.parent_mounts_child.as_str(),
        ),
        (
            PLUGIN_IMPORTER_D13_REVIEW_CHILDREN_CHILD,
            sources.review_children_child.as_str(),
        ),
        (
            PLUGIN_IMPORTER_D13_BUDGETS_CHILD,
            sources.budgets_child.as_str(),
        ),
        (
            PLUGIN_IMPORTER_D13_STATUS_MIRRORS_CHILD,
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
fn runtime_15_plugin_importer_d13_sdk_structure_assertions_children_line_budgets_are_current() {
    let sources = plugin_importer_d13_sdk_structure_sources();

    assert_plugin_importer_d13_sdk_structure_assertions_children_line_budgets_are_current(&sources);
}
