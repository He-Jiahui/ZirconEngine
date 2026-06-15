use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::ZrPackPromotionMethod;
use crate::asset::pack::ZrPackDocumentManifest;

pub const ZRPACK_INSTALL_RECEIPT_FORMAT_VERSION: u32 = 2;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZrPackInstallReceipt {
    pub format_version: u32,
    pub base_pack: PathBuf,
    pub delta_pack: PathBuf,
    pub staged_pack: PathBuf,
    pub installed_pack: PathBuf,
    pub backup_pack: Option<PathBuf>,
    pub target_manifest: ZrPackDocumentManifest,
    pub installed_manifest: ZrPackDocumentManifest,
    pub staged_size: u64,
    pub installed_size: u64,
    pub delta_apply_verified: bool,
    pub promotion_method: ZrPackPromotionMethod,
    pub promoted: bool,
}
