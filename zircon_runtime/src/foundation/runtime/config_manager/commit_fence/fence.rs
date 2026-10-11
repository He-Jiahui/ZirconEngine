//! 配置路径 gate 与 epoch 约束最终提交；检查时已取消或已被新 epoch 替代的 worker 不执行提交闭包。
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use super::registry::{lock, reclaim_path_gate, register_path_gate, PathCommitEpoch};

/// 绑定词法路径 gate、激活 epoch 和取消状态的最终文件提交准入对象。
///
/// # 使用时机
///
/// 恢复读取和最终发布都通过 [Self::commit] 与同一路径的新激活互斥。
/// 新 epoch 使旧对象的后续提交失效；[Self::cancel] 阻止后续准入。
///
/// # 生命周期
///
/// 闭包获准后仍持有 gate；取消不会撤回已获准的闭包。
/// Arc 共享对象的最后一次释放尝试回收 registry 条目，不能当作文件已持久化的回执。
pub(in crate::foundation::runtime) struct ConfigCommitFence {
    path: PathBuf,
    epoch: u64,
    gate: Arc<Mutex<PathCommitEpoch>>,
    cancelled: AtomicBool,
    commit_active: AtomicBool,
}

impl ConfigCommitFence {
    /// 取得新激活的路径提交身份，供 worker 创建前使用。
    ///
    /// # 参数与返回
    ///
    /// `path` 是实际文件路径；返回共享 fence，或在 gate 占用、epoch 耗尽时返回注册错误。
    /// 路径键归一化只用于进程内 registry，不改写实际 I/O 路径。
    ///
    /// # 参考
    ///
    /// 注册规则见 [register_path_gate]，后续发布使用 [Self::commit]。
    pub(in crate::foundation::runtime::config_manager) fn register(
        path: &Path,
    ) -> io::Result<Arc<Self>> {
        let (path, epoch, gate) = register_path_gate(path)?;
        Ok(Arc::new(Self {
            path,
            epoch,
            gate,
            cancelled: AtomicBool::new(false),
            commit_active: AtomicBool::new(false),
        }))
    }

    // gate 锁覆盖 epoch 检查与文件提交闭包，阻止新 epoch 交错；cancel 只设置原子标记，已通过检查的提交仍可能完成。
    /// 在路径 gate 内检查准入并同步执行文件操作。
    ///
    /// # 参数
    ///
    /// - `commit`：实际文件操作；只有当前 epoch 且未取消时才执行，闭包期间一直持有 gate。
    ///
    /// # 返回与错误
    ///
    /// 返回闭包结果。准入失败返回 Interrupted，闭包不执行；操作错误原样传播。
    /// 成功只代表此闭包操作成功，具体持久化保证由调用者选择的文件操作承担。
    ///
    /// # 生命周期与参考
    ///
    /// 已获准操作不会因随后 [Self::cancel] 而被撤回。
    /// 正常返回、提前返回和 panic 均经 [CommitActiveGuard] 清除诊断 active 标志。
    pub(in crate::foundation::runtime) fn commit<T>(
        &self,
        commit: impl FnOnce() -> io::Result<T>,
    ) -> io::Result<T> {
        let state = lock(&self.gate);
        self.commit_active.store(true, Ordering::Release);
        let active = CommitActiveGuard(&self.commit_active);
        if self.cancelled.load(Ordering::Acquire) || state.current != self.epoch {
            return Err(io::Error::new(
                io::ErrorKind::Interrupted,
                format!(
                    "config commit for {} was cancelled or superseded",
                    self.path.display()
                ),
            ));
        }
        let result = commit();
        drop(active);
        result
    }

    /// 阻止本 fence 的后续文件操作获准，供关闭预算耗尽路径使用。
    ///
    /// # 返回与生命周期
    ///
    /// 返回读取时的 commit_active 标志，仅供关闭日志判断是否有提交作用域在途。
    /// 取消不等待闭包退出、不撤回已获准操作，返回值也不是停止或持久化回执。
    ///
    /// # 参考
    ///
    /// 提交行为见 [Self::commit]；worker 关闭后仍须回收线程。
    pub(in crate::foundation::runtime::config_manager) fn cancel(&self) -> bool {
        self.cancelled.store(true, Ordering::Release);
        self.commit_active.load(Ordering::Acquire)
    }
}

impl fmt::Debug for ConfigCommitFence {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ConfigCommitFence")
            .field("path", &self.path)
            .field("epoch", &self.epoch)
            .field("cancelled", &self.cancelled.load(Ordering::Acquire))
            .field("commit_active", &self.commit_active.load(Ordering::Acquire))
            .finish_non_exhaustive()
    }
}

impl Drop for ConfigCommitFence {
    fn drop(&mut self) {
        reclaim_path_gate(&self.path, &self.gate);
    }
}

/// 在提交作用域结束时清除 active 诊断标志。
///
/// # 生命周期
///
/// 借用 fence 的原子标志，正常返回、提前返回与闭包 panic 都执行 Drop。
/// path gate 由 [ConfigCommitFence::commit] 的锁守卫持有，释放 gate 不由此 guard 承担。
struct CommitActiveGuard<'a>(&'a AtomicBool);

impl Drop for CommitActiveGuard<'_> {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}
