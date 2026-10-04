use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::asset::pack::ZrPackDocumentManifest;

/// 安装回执保存的发布方式；当前事务返回 AtomicReplacement 或 AlreadyInstalled，其余变体保留在序列化格式中。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZrPackPromotionMethod {
    AtomicReplacement,
    AlreadyInstalled,
    Renamed,
    CopiedAfterRenameFailure,
}

/// promotion 成功后的路径、manifest 和大小快照；receipt 使用它证明最终发布结果。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ZrPackPromotionReport {
    pub installed_pack: PathBuf,
    pub backup_pack: Option<PathBuf>,
    pub staged_pack: PathBuf,
    pub installed_manifest: ZrPackDocumentManifest,
    pub installed_size: u64,
    pub promotion_method: ZrPackPromotionMethod,
}
