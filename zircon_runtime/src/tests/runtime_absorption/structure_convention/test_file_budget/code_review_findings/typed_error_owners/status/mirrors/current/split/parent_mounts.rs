use super::super::super::super::super::super::super::*;
use super::super::super::super::TYPED_ERROR_STATUS_DOCS_STATUS_MIRRORS_STATUS_CURRENT_CHILD;

#[test]
fn runtime_15_typed_error_status_doc_status_mirrors_status_current_is_child_backed() {
    assert_typed_error_status_mirror_status_current_parent_mounts_children();
}

pub(super) fn assert_typed_error_status_mirror_status_current_parent_mounts_children() {
    let parent = read_runtime_src(TYPED_ERROR_STATUS_DOCS_STATUS_MIRRORS_STATUS_CURRENT_CHILD);

    assert_contains_all(
        "typed-error status-doc status mirrors status-current parent mounts focused children",
        &parent,
        &[
            "#[path = \"current/ownership.rs\"]",
            "mod ownership;",
            "#[path = \"current/sources.rs\"]",
            "mod sources;",
            "#[path = \"current/split_layout.rs\"]",
            "mod split_layout;",
            "#[path = \"current/status_sync.rs\"]",
            "mod status_sync;",
            "pub(super) use ownership::*;",
            "pub(super) use sources::*;",
        ],
    );
    super::parent_backflow::assert_typed_error_status_mirror_status_current_parent_has_no_moved_checks(
        &parent,
    );
    super::child_inventory::assert_typed_error_status_mirror_status_current_direct_child_inventory(
    );
    super::status_mirrors::assert_typed_error_status_mirror_status_current_status_is_current();
    super::budgets::assert_typed_error_status_mirror_status_current_children_line_budgets(&parent);
}
