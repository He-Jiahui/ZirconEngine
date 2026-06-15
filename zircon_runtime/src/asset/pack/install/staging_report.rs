use std::path::PathBuf;

use crate::asset::pack::ZrPackDocumentManifest;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ZrPackDeltaInstallReport {
    pub base_pack: PathBuf,
    pub delta_pack: PathBuf,
    pub staged_pack: PathBuf,
    pub target_manifest: ZrPackDocumentManifest,
    pub staged_size: u64,
    pub delta_apply_verified: bool,
}
