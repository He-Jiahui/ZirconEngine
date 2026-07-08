use super::super::super::super::super::super::*;
use super::super::super::*;
use super::super::*;

pub(in super::super) fn assert_typed_error_status_maps_are_child_backed() {
    let parent = read_runtime_src(TYPED_ERROR_STATUS_DOCS_STATUS_MAPS_CHILD);
    let child_tree = super::sources::typed_error_status_map_child_source_blob();

    assert_contains_all(
        "typed-error status-doc status maps parent mounts child owners",
        &parent,
        &[
            "#[path = \"maps/child_inventory.rs\"]",
            "mod child_inventory;",
            "#[path = \"maps/review_slices.rs\"]",
            "mod review_slices;",
            "#[path = \"maps/status_current.rs\"]",
            "mod status_current;",
            "pub(super) use child_inventory::*;",
            "assert_typed_error_status_maps_are_synced",
            TYPED_ERROR_STATUS_DOCS_STATUS_MAPS_OWNERSHIP_GUARD,
        ],
    );
    for moved_anchor in [
        "status/date expected-slice maps",
        "Runtime 15 M3 code review findings typed-error structure guard child-owner split",
        "runtime_15_native_live_host_typed_error_review_guard_child_owner_split_static_passed_cargo_deferred",
        "runtime_15_shader_prewarm_cli_typed_error_review_guard_child_owner_split_static_passed_cargo_deferred",
        "2026-06-30",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "typed-error status-doc status map anchor `{moved_anchor}` should stay in child files"
        );
    }
    for (_, child_path, anchor) in TYPED_ERROR_STATUS_DOCS_STATUS_MAP_CHILDREN {
        assert!(
            child_tree.contains(child_path),
            "typed-error status-doc status map child tree should inventory child path {child_path}"
        );
        assert!(
            child_tree.contains(anchor),
            "typed-error status-doc status map child {child_path} should own anchor {anchor}"
        );
    }

    super::status_sync::assert_typed_error_status_maps_status_is_current();

    for (path, source) in [(TYPED_ERROR_STATUS_DOCS_STATUS_MAPS_CHILD, parent)]
        .into_iter()
        .chain(super::sources::typed_error_status_map_child_sources())
    {
        let line_count = source.lines().count();
        assert!(
            line_count < TYPED_ERROR_CHILD_OWNER_LINE_BUDGET,
            "{path} should stay below the Runtime 15 status-doc status-map budget; got {line_count} lines"
        );
    }
}
