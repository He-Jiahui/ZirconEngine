use super::*;

#[test]
fn runtime_15_status_output_runtime_15_expected_slice_child_owner_budget_sources_are_folder_backed()
{
    let parent = read_runtime_src(&format!("tests/runtime_absorption/{BUDGETS_SOURCES_PATH}"));
    let children = BUDGETS_SOURCES_CHILDREN
        .iter()
        .map(|path| read_runtime_src(&format!("tests/runtime_absorption/{path}")))
        .collect::<Vec<_>>()
        .join("\n");

    assert_contains_all(
        "child-owner budget source inventory parent",
        &parent,
        &[
            "#[path = \"sources/budgets.rs\"]",
            "mod budgets;",
            "#[path = \"sources/doc_mirrors.rs\"]",
            "mod doc_mirrors;",
            "#[path = \"sources/folder_backed.rs\"]",
            "mod folder_backed;",
            "#[path = \"sources/metadata.rs\"]",
            "mod metadata;",
            "#[path = \"sources/source_paths.rs\"]",
            "mod source_paths;",
            "#[path = \"sources/status_mirrors.rs\"]",
            "mod status_mirrors;",
            "pub(super) use metadata::*;",
            "pub(super) use source_paths::*;",
        ],
    );
    for moved_anchor in [
        "pub(super) const BUDGET_SLICE",
        "EXPECTED_SLICE_BUDGET_SOURCE_PATHS",
        "plan_status/status_output_tables/expected_slices/status/runtime_15.rs",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "budgets/sources.rs should delegate moved source inventory anchor {moved_anchor}"
        );
    }
    assert_contains_all(
        "child-owner budget source inventory children",
        &children,
        &[
            BUDGET_SLICE,
            BUDGET_STATUS,
            BUDGET_SOURCE_SLICE,
            BUDGET_SOURCE_STATUS,
            BUDGET_SOURCE_GUARD,
            "EXPECTED_SLICE_BUDGET_SOURCE_PATHS",
            "status/runtime_15/m3_structure_support/status_support_maps.rs",
            "date/runtime_15/m3_structure_support/status_support_maps.rs",
        ],
    );
}
