#[path = "../../../support/options/state/extract/assembly.rs"]
mod assembly;
#[path = "../../../support/options/state/extract/entry.rs"]
mod entry;
#[path = "../../../support/options/state/extract/key.rs"]
mod key;
#[path = "../../../support/options/state/extract/required.rs"]
mod required;

pub(in super::super) use entry::take_option_manifest_signature;
