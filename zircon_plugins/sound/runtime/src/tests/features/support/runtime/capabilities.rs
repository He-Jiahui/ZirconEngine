#[path = "../../../support/runtime/capabilities/entry.rs"]
mod entry;
#[path = "../../../support/runtime/capabilities/ordering.rs"]
mod ordering;
#[path = "../../../capabilities/support_projection.rs"]
mod projection;

pub(super) use entry::capability_signatures;
