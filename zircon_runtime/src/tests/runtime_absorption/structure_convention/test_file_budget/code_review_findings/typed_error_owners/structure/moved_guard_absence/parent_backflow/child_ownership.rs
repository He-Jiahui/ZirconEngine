use super::super::super::super::super::super::*;
use super::super::super::TYPED_ERROR_CHILD_OWNER_LINE_BUDGET;
use super::super::*;
use super::*;

#[test]
fn runtime_15_typed_error_moved_guard_absence_parent_backflow_is_child_backed() {
    let parent = read_runtime_src(TYPED_ERROR_STRUCTURE_MOVED_GUARD_ABSENCE_PARENT_BACKFLOW_CHILD);
    let child_tree = typed_error_parent_backflow_child_source_blob();

    assert_contains_all(
        "typed-error moved-guard parent-backflow parent mounts focused children",
        &parent,
        &[
            "#[path = \"parent_backflow/child_inventory.rs\"]",
            "mod child_inventory;",
            "#[path = \"parent_backflow/child_ownership.rs\"]",
            "mod child_ownership;",
            "#[path = \"parent_backflow/guard_body.rs\"]",
            "mod guard_body;",
            "#[path = \"parent_backflow/guard_names.rs\"]",
            "mod guard_names;",
            "#[path = \"parent_backflow/metadata.rs\"]",
            "mod metadata;",
            "#[path = \"parent_backflow/parent_paths.rs\"]",
            "mod parent_paths;",
            "#[path = \"parent_backflow/sources.rs\"]",
            "mod sources;",
            "pub(super) use child_inventory::*;",
            "pub(super) use guard_body::*;",
            "pub(super) use guard_names::*;",
            "pub(super) use metadata::*;",
            "pub(super) use parent_paths::*;",
            "pub(super) use sources::*;",
        ],
    );
    for moved_anchor in [
        "const TYPED_ERROR_PARENT_PATHS",
        "const PARENT_BACKFLOW_GUARDS",
        "let parent_sources = TYPED_ERROR_PARENT_PATHS",
        "for child_owned_test in PARENT_BACKFLOW_GUARDS",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "typed-error parent-backflow helper `{moved_anchor}` should stay in focused child files"
        );
    }
    for (_, child_path, anchor) in TYPED_ERROR_MOVED_GUARD_ABSENCE_PARENT_BACKFLOW_CHILDREN {
        assert!(
            child_tree.contains(child_path),
            "typed-error parent-backflow tree should inventory child path {child_path}"
        );
        assert!(
            child_tree.contains(anchor),
            "typed-error parent-backflow child {child_path} should own anchor {anchor}"
        );
    }
    for (path, source) in [(
        TYPED_ERROR_STRUCTURE_MOVED_GUARD_ABSENCE_PARENT_BACKFLOW_CHILD,
        parent,
    )]
    .into_iter()
    .chain(typed_error_parent_backflow_child_sources())
    {
        let line_count = source.lines().count();
        assert!(
            line_count < TYPED_ERROR_CHILD_OWNER_LINE_BUDGET,
            "{path} should stay below the Runtime 15 parent-backflow child budget; got {line_count} lines"
        );
    }

    assert_typed_error_parent_backflow_guards_are_absent();
}
