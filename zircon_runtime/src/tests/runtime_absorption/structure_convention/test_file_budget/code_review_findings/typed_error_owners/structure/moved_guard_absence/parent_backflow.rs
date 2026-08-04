use super::super::super::super::super::*;
use super::*;

#[path = "parent_backflow/child_inventory.rs"]
mod child_inventory;
#[path = "parent_backflow/child_ownership.rs"]
mod child_ownership;
#[path = "parent_backflow/guard_body.rs"]
mod guard_body;
#[path = "parent_backflow/guard_names.rs"]
mod guard_names;
#[path = "parent_backflow/metadata.rs"]
mod metadata;
#[path = "parent_backflow/parent_paths.rs"]
mod parent_paths;
#[path = "parent_backflow/sources.rs"]
mod sources;

pub(super) use child_inventory::*;
pub(super) use guard_body::*;
pub(super) use guard_names::*;
pub(super) use metadata::*;
pub(super) use parent_paths::*;
pub(super) use sources::*;

#[test]
fn runtime_15_typed_error_moved_guard_absence_parent_backflow_guards_are_child_owned() {
    assert_typed_error_parent_backflow_guards_are_absent();
}
