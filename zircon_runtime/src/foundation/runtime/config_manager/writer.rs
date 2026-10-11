use std::io;
use std::path::Path;

use crate::core::resource::io::stage_atomic_write;

use super::commit_fence::ConfigCommitFence;

/// 发布一份配置 JSON 快照的实现边界，供后台 worker 和可控 fixture 使用。
///
/// # 实现约束
///
/// 暂存可在 path gate 外完成；最终目标发布必须通过传入的 [ConfigCommitFence]。
/// 只完成暂存不能报告发布成功；取消或旧 epoch 被拒绝后，不得继续替换目标。
///
/// # 生命周期与参考
///
/// 实现须可由后台线程共享调用，并在返回前结束本次发布操作。
/// 参见 [Self::write] 和 [ConfigCommitFence::commit]。
pub(in crate::foundation::runtime) trait ConfigFileWriter:
    Send + Sync
{
    /// 按指定 fence 发布整份快照。
    ///
    /// # 参数
    ///
    /// - `path`：目标文件的实际 I/O 路径。
    /// - `bytes`：已经序列化的完整 JSON；借用只在本次调用期间有效。
    /// - `commit_fence`：最终发布的取消与 epoch 准入边界。
    ///
    /// # 返回与错误
    ///
    /// 成功应表示最终发布已结束，不能仅表示排队或暂存完成。
    /// 返回暂存、准入或发布错误，由 worker 记录并决定后续重试。
    ///
    /// # 参考
    ///
    /// 最终发布使用 [ConfigCommitFence::commit]。
    fn write(&self, path: &Path, bytes: &[u8], commit_fence: &ConfigCommitFence) -> io::Result<()>;
}

#[derive(Debug, Default)]
pub(super) struct AtomicConfigFileWriter;

impl ConfigFileWriter for AtomicConfigFileWriter {
    fn write(&self, path: &Path, bytes: &[u8], commit_fence: &ConfigCommitFence) -> io::Result<()> {
        let pending = stage_atomic_write(path, bytes)?;
        commit_fence.commit(|| pending.commit())
    }
}
