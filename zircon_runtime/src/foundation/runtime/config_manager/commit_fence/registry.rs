use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock, TryLockError, Weak};

use super::path_key::absolute_path;

static PATH_COMMIT_GATES: OnceLock<Mutex<HashMap<PathBuf, Weak<Mutex<PathCommitEpoch>>>>> =
    OnceLock::new();

#[derive(Debug, Default)]
pub(super) struct PathCommitEpoch {
    pub(super) current: u64,
}

/// 为同一进程内的词法路径键注册新 epoch，供新配置服务取代尚未获准的旧提交。
///
/// # 参数
///
/// - `path`：实际文件路径；registry 只使用其词法归一化键，不改变 writer 的 I/O 参数。
///
/// # 返回与错误
///
/// 返回路径键、新 epoch 和共享 gate。成功递增 epoch 后，旧 fence 的后续提交会被拒绝。
/// gate 正被占用时返回 WouldBlock；epoch 耗尽时返回错误，不复用旧身份。
///
/// # 生命周期与参考
///
/// registry 保留 Weak，活跃 fence 共享 gate；最后一个 fence 释放后由 [reclaim_path_gate] 清理。
/// 这里不解析文件系统别名，也不提供跨进程锁。
pub(super) fn register_path_gate(
    path: &Path,
) -> io::Result<(PathBuf, u64, Arc<Mutex<PathCommitEpoch>>)> {
    let path = absolute_path(path);
    let gates = PATH_COMMIT_GATES.get_or_init(|| Mutex::new(HashMap::new()));
    let mut gates = lock(gates);
    let gate = gates.get(&path).and_then(Weak::upgrade).unwrap_or_else(|| {
        let gate = Arc::new(Mutex::new(PathCommitEpoch::default()));
        gates.insert(path.clone(), Arc::downgrade(&gate));
        gate
    });
    let mut state = match gate.try_lock() {
        Ok(state) => state,
        Err(TryLockError::Poisoned(poisoned)) => poisoned.into_inner(),
        Err(TryLockError::WouldBlock) => {
            return Err(io::Error::new(
                io::ErrorKind::WouldBlock,
                format!(
                    "a config filesystem commit for {} is still in progress",
                    path.display()
                ),
            ));
        }
    };
    state.current = state.current.checked_add(1).ok_or_else(|| {
        io::Error::other(format!(
            "config filesystem commit epoch exhausted for {}",
            path.display()
        ))
    })?;
    let epoch = state.current;
    drop(state);
    drop(gates);
    Ok((path, epoch, gate))
}

pub(super) fn reclaim_path_gate(path: &Path, gate: &Arc<Mutex<PathCommitEpoch>>) {
    if Arc::strong_count(gate) != 1 {
        return;
    }
    let Some(gates) = PATH_COMMIT_GATES.get() else {
        return;
    };
    // 等待 registry 锁时其他注册可能升级 Weak，拿锁后须再验唯一 owner。
    // 还要核对 Arc 身份，避免旧 gate 的析构误删后来替换的同路径条目。
    let mut gates = lock(gates);
    if Arc::strong_count(gate) != 1 {
        return;
    }
    let owns_entry = gates
        .get(path)
        .is_some_and(|registered| registered.as_ptr() == Arc::as_ptr(gate));
    if owns_entry {
        gates.remove(path);
    }
}

pub(super) fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(test)]
pub(super) fn contains_path(path: &Path) -> bool {
    lock(PATH_COMMIT_GATES.get_or_init(|| Mutex::new(HashMap::new()))).contains_key(path)
}
