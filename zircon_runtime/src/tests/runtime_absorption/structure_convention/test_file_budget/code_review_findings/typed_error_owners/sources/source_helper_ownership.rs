use super::super::super::super::*;
use super::*;

#[test]
fn runtime_15_typed_error_source_inventory_source_helpers_are_child_backed() {
    let parent = read_runtime_src(TYPED_ERROR_SOURCE_INVENTORY_CHILD);

    assert_contains_all(
        "typed-error source inventory parent mounts source helper children",
        &parent,
        &[
            "#[path = \"sources/child_sources.rs\"]",
            "mod child_sources;",
            "#[path = \"sources/child_inventory.rs\"]",
            "mod child_inventory;",
            "#[path = \"sources/metadata.rs\"]",
            "mod metadata;",
            "#[path = \"sources/source_helper_ownership.rs\"]",
            "mod source_helper_ownership;",
            "#[path = \"sources/source_helper_status.rs\"]",
            "mod source_helper_status;",
            "pub(super) use child_sources::*;",
            "pub(super) use child_inventory::*;",
            "pub(super) use metadata::*;",
        ],
    );
    let mut budget_sources: Vec<(&'static str, String)> =
        vec![(TYPED_ERROR_SOURCE_INVENTORY_CHILD, parent)];

    for (path, source) in budget_sources {
        let line_count = source.lines().count();
        assert!(
            line_count < TYPED_ERROR_CHILD_OWNER_LINE_BUDGET,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
}
