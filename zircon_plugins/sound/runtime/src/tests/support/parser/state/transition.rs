#[path = "../../../state/transition/support_dependency.rs"]
mod dependency;
#[path = "transition/entry.rs"]
mod entry;
#[path = "../../../state/transition/support_feature.rs"]
mod feature;
#[path = "transition/module.rs"]
mod module;
#[path = "../../../state/transition/support_pending_rows.rs"]
mod pending_rows;

pub(super) use entry::enter_section;
