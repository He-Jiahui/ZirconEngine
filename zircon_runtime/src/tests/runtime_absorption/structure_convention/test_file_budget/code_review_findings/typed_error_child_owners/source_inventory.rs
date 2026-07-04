use super::super::super::*;

#[path = "source_inventory/budgets.rs"]
mod budgets;
#[path = "source_inventory/child_inventory.rs"]
mod child_inventory;
#[path = "source_inventory/child_sources.rs"]
mod child_sources;
#[path = "source_inventory/delegation.rs"]
mod delegation;
#[path = "source_inventory/metadata.rs"]
mod metadata;
#[path = "source_inventory/paths.rs"]
mod paths;
#[path = "source_inventory/reads.rs"]
mod reads;
#[path = "source_inventory/source_helper_ownership.rs"]
mod source_helper_ownership;
#[path = "source_inventory/source_helper_status.rs"]
mod source_helper_status;
#[path = "source_inventory/status_mirrors.rs"]
mod status_mirrors;

pub(super) use child_inventory::*;
pub(super) use child_sources::*;
pub(super) use metadata::*;

pub(super) fn typed_error_children_source() -> String {
    reads::typed_error_children_source()
}

pub(super) fn assert_typed_error_line_budgets() {
    budgets::assert_typed_error_line_budgets();
}

pub(super) fn typed_error_review_guard_count() -> usize {
    reads::typed_error_review_guard_count()
}

#[test]
fn runtime_15_typed_error_source_inventory_is_child_owner() {
    let sources = typed_error_source_inventory_sources();

    delegation::assert_typed_error_source_inventory_is_child_owner(&sources);
    budgets::assert_typed_error_source_inventory_children_line_budgets_are_current(&sources);
}
