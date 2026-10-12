use super::{RuntimeSessionArchiveError, RuntimeSessionArchiveManifest};

/// 单次检查会话存档路径得到的三态结果，用于区分首次创建、展示槽位摘要和报告读取错误。
/// 结果不保留文件锁或写入权限；后续操作仍须重新读取并处理路径变化。
#[derive(Debug)]
pub enum RuntimeSessionArchivePathStatus {
    /// 目标路径缺失，捕获流程可以从空存档开始。
    Missing,
    /// 存档已成功读取并验证；摘要对应本次检查时的内容。
    Available {
        manifest: RuntimeSessionArchiveManifest,
    },
    Invalid {
        error: RuntimeSessionArchiveError,
    },
}

impl RuntimeSessionArchivePathStatus {
    /// 仅识别可按“尚未创建存档”处理的缺失状态；其他读取错误不算缺失。
    pub fn is_missing(&self) -> bool {
        matches!(self, Self::Missing)
    }

    /// 返回本次检查的有效摘要；若要反映路径后续变化，调用方须再次检查。
    pub fn manifest(&self) -> Option<&RuntimeSessionArchiveManifest> {
        match self {
            Self::Available { manifest } => Some(manifest),
            Self::Missing | Self::Invalid { .. } => None,
        }
    }
}
