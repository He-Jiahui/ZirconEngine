use super::super::super::super::super::*;
use super::super::*;

pub(super) fn assert_typed_error_source_inventory_parent_mounts_focused_owners(
    sources: &TypedErrorSourceInventorySources,
) {
    assert_contains_all(
        "typed-error source inventory child mounts focused owners",
        &sources.source_inventory_child,
        &[
            "#[path = \"sources/budgets.rs\"]",
            "mod budgets;",
            "#[path = \"sources/child_sources.rs\"]",
            "mod child_sources;",
            "#[path = \"sources/child_inventory.rs\"]",
            "mod child_inventory;",
            "#[path = \"sources/delegation.rs\"]",
            "mod delegation;",
            "#[path = \"sources/metadata.rs\"]",
            "mod metadata;",
            "#[path = \"sources/paths.rs\"]",
            "mod paths;",
            "#[path = \"sources/reads.rs\"]",
            "mod reads;",
            "#[path = \"sources/source_helper_ownership.rs\"]",
            "mod source_helper_ownership;",
            "#[path = \"sources/source_helper_status.rs\"]",
            "mod source_helper_status;",
            "#[path = \"sources/status_mirrors.rs\"]",
            "mod status_mirrors;",
            "pub(super) use child_sources::*;",
            "pub(super) use child_inventory::*;",
            "pub(super) use metadata::*;",
            "reads::typed_error_children_source",
            "budgets::assert_typed_error_line_budgets",
            "reads::typed_error_review_guard_count",
            "runtime_15_typed_error_source_inventory_is_child_owner",
        ],
    );
}
