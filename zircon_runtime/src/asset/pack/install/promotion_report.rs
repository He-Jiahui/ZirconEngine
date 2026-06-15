use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::asset::pack::ZrPackDocumentManifest;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZrPackPromotionMethod {
    Renamed,
    CopiedAfterRenameFailure,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ZrPackPromotionReport {
    pub installed_pack: PathBuf,
    pub backup_pack: Option<PathBuf>,
    pub staged_pack: PathBuf,
    pub installed_manifest: ZrPackDocumentManifest,
    pub installed_size: u64,
    pub promotion_method: ZrPackPromotionMethod,
}
