#[path = "../manifest/assembly.rs"]
mod assembly;
#[path = "../manifest/attachments.rs"]
mod attachments;
#[path = "../../../support/options/state/manifest/base.rs"]
mod base;

pub(super) use assembly::option_manifest_from_pending;
