#[path = "../../../support/runtime/dependencies/entry.rs"]
mod entry;
#[path = "../../../support/runtime/dependencies/ordering.rs"]
mod ordering;
#[path = "../../../dependencies/support_projection.rs"]
mod projection;
#[path = "../../../support/runtime/dependencies/signature.rs"]
mod signature;

pub(super) use entry::dependency_signatures;
