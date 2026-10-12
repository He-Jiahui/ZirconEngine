#[path = "../signature/extract.rs"]
mod extract;
#[path = "../../../support/options/state/signature/record.rs"]
mod record;

pub(super) use extract::take_option_manifest_signature;
pub(super) use record::OptionManifestSignature;
