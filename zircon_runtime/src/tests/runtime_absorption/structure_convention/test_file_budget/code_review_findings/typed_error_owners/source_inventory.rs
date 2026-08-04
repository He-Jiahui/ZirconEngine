use super::super::super::*;
use super::TYPED_ERROR_CHILD_OWNER_LINE_BUDGET;

#[path = "sources/budgets.rs"]
mod budgets;
#[path = "sources/child_inventory.rs"]
mod child_inventory;
#[path = "sources/child_sources.rs"]
mod child_sources;
#[path = "sources/delegation.rs"]
mod delegation;
#[path = "sources/metadata.rs"]
mod metadata;
#[path = "sources/paths.rs"]
mod paths;
#[path = "sources/reads.rs"]
mod reads;
#[path = "sources/source_helper_ownership.rs"]
mod source_helper_ownership;

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
