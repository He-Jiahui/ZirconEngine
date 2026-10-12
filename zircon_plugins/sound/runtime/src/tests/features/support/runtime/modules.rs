#[path = "modules/entry.rs"]
mod entry;
#[path = "../../../support/runtime/modules/ordering.rs"]
mod ordering;
#[path = "../../../support/runtime/modules/projection.rs"]
mod projection;
#[path = "../../../support/runtime/modules/signature.rs"]
mod signature;

pub(super) use entry::module_signatures;
