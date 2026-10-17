//! 配置路径 gate 与 epoch 约束最终提交；检查时已取消或已被新 epoch 替代的 worker 不执行提交闭包。
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use crate::asset::project::ResolvedProjectPathIdentity;
use crate::core::{CoreError, CoreWeak};
use crate::foundation::runtime::config_manager::config_error;

use super::path_key::physical_path;
use super::registry::{
    initialize_path_gate, lock, reclaim_path_gate, reserve_path_owner, ConfigOwnerLease,
    PathCommitEpoch,
};

/// 绑定物理路径、独占 owner lease、激活 epoch 和取消状态的最终提交准入对象。
///
/// # 使用时机
///
/// [Self::initialize_owned] 的恢复读取和 [Self::commit] 的最终发布使用同一路径 gate 互斥。
/// 新 epoch 使旧对象的后续提交失效；[Self::cancel] 阻止后续准入。
///
/// # 生命周期
///
/// 闭包获准后仍持有 gate；取消不会撤回已获准的闭包。
/// Arc 共享对象的最后一次释放尝试回收 registry 条目，不能当作文件已持久化的回执。
pub(in crate::foundation::runtime) struct ConfigCommitFence {
    path: PathBuf,
    identity: ResolvedProjectPathIdentity,
    epoch: u64,
    gate: Arc<Mutex<PathCommitEpoch>>,
    cancelled: AtomicBool,
    commit_active: AtomicBool,
    // Fields drop in declaration order: release the final gate before the owner lease.
    _owner_lease: Option<Arc<ConfigOwnerLease>>,
}

impl ConfigCommitFence {
    /// Reserve the physical path before recovery, parsing, or worker creation.
    /// Both same-Core and different-Core independent managers conflict while the lease lives.
    /// Failed preparation cancels the candidate and releases its reservation after gate unlock.
    pub(in crate::foundation::runtime::config_manager) fn initialize_owned<T>(
        path: &Path,
        core: CoreWeak,
        initialize: impl FnOnce(Arc<Self>) -> Result<T, CoreError>,
    ) -> Result<T, CoreError> {
        let context = path.to_path_buf();
        let identity =
            physical_path(path).map_err(|error| config_error(&context, error.to_string()))?;
        let mut owner = reserve_path_owner(&identity, core)?;
        let result =
            Self::initialize_resolved(identity, Some(Arc::clone(&owner.lease)), initialize)
                .map_err(|error| config_error(&context, error.to_string()))??;
        owner.prepared = true;
        Ok(result)
    }

    /// Low-level epoch fixture entry; production construction always reserves an owner.
    #[cfg(test)]
    pub(in crate::foundation::runtime::config_manager) fn initialize<T, E>(
        path: &Path,
        initialize: impl FnOnce(Arc<Self>) -> Result<T, E>,
    ) -> io::Result<Result<T, E>> {
        Self::initialize_resolved(physical_path(path)?, None, initialize)
    }

    fn initialize_resolved<T, E>(
        identity: ResolvedProjectPathIdentity,
        owner_lease: Option<Arc<ConfigOwnerLease>>,
        initialize: impl FnOnce(Arc<Self>) -> Result<T, E>,
    ) -> io::Result<Result<T, E>> {
        initialize_path_gate(identity, |identity, epoch, gate| {
            let fence = Arc::new(Self {
                path: identity.operation_path().to_path_buf(),
                identity,
                _owner_lease: owner_lease,
                epoch,
                gate,
                cancelled: AtomicBool::new(false),
                commit_active: AtomicBool::new(false),
            });
            let mut pending = PendingFenceGuard {
                fence: Arc::clone(&fence),
                prepared: false,
            };
            let result = initialize(fence);
            pending.prepared = result.is_ok();
            result
        })
    }

    pub(in crate::foundation::runtime::config_manager) fn operation_path(&self) -> &Path {
        &self.path
    }

    #[cfg(test)]
    pub(in crate::foundation::runtime::config_manager) fn register(
        path: &Path,
    ) -> io::Result<Arc<Self>> {
        Self::initialize(path, |fence| Ok(fence))?
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
        reclaim_path_gate(&self.identity, &self.gate);
    }
}

// An unsuccessful candidate may have escaped through a clone. Cancel it before releasing
// the gate so a later attempt reusing the uncommitted epoch cannot authorize that clone.
struct PendingFenceGuard {
    fence: Arc<ConfigCommitFence>,
    prepared: bool,
}

impl Drop for PendingFenceGuard {
    fn drop(&mut self) {
        if !self.prepared {
            self.fence.cancel();
        }
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
