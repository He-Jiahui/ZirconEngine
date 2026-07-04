use super::super::super::super::super::*;
use super::super::*;

pub(super) fn assert_typed_error_source_inventory_parent_mounts_focused_owners(
    sources: &TypedErrorSourceInventorySources,
) {
    assert_contains_all(
        "typed-error source inventory child mounts focused owners",
        &sources.source_inventory_child,
        &[
            "#[path = \"source_inventory/budgets.rs\"]",
            "mod budgets;",
            "#[path = \"source_inventory/child_sources.rs\"]",
            "mod child_sources;",
            "#[path = \"source_inventory/child_inventory.rs\"]",
            "mod child_inventory;",
            "#[path = \"source_inventory/delegation.rs\"]",
            "mod delegation;",
            "#[path = \"source_inventory/metadata.rs\"]",
            "mod metadata;",
            "#[path = \"source_inventory/paths.rs\"]",
            "mod paths;",
            "#[path = \"source_inventory/reads.rs\"]",
            "mod reads;",
            "#[path = \"source_inventory/source_helper_ownership.rs\"]",
            "mod source_helper_ownership;",
            "#[path = \"source_inventory/source_helper_status.rs\"]",
            "mod source_helper_status;",
            "#[path = \"source_inventory/status_mirrors.rs\"]",
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
