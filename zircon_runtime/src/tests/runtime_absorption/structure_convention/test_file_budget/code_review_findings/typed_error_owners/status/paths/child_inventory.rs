#[path = "inventory/guard_children.rs"]
mod guard_children;
#[path = "inventory/paths_children.rs"]
mod paths_children;
#[path = "inventory/source_helper_children.rs"]
mod source_helper_children;
#[path = "inventory/split_layout.rs"]
mod split_layout;
#[path = "inventory/status_current_children.rs"]
mod status_current_children;

pub(in super::super) use guard_children::TYPED_ERROR_STATUS_DOCS_GUARD_CHILDREN;
pub(in super::super) use paths_children::TYPED_ERROR_STATUS_DOCS_PATHS_CHILDREN;
pub(in super::super) use source_helper_children::TYPED_ERROR_STATUS_DOCS_SOURCE_HELPER_CHILDREN;
pub(in super::super) use status_current_children::TYPED_ERROR_STATUS_DOCS_PATHS_STATUS_CURRENT_CHILDREN;
