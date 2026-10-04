//! 先发现并验证整批证据，再恢复或清理；提交与恢复共享 owner 锁，避免并发改写同一日志目录。
//! Restart recovery for durable transaction journals.

mod discovery;
mod evidence;
mod policy;
mod replay;
mod validation;

use std::path::{Path, PathBuf};

use discovery::load_pending_transactions;
use replay::recover_journal;
use validation::{operation, resolve_recovery_directory};

use super::error::{DurableTransactionError, TransactionPhase};
use super::journal::truncate_torn_tail;
use super::observation::DurableRecoveryReport;
use super::owner_lock::TransactionOwnerLock;
use super::stage::remove_reserved_if_exists;

pub use policy::{RecoveryMode, RecoveryPolicy};

/// 验证并列出待恢复证据，不执行回放；仍会获取 owner 锁，必要时创建持久锁文件。
/// 适合命令行预检，但调用端必须具备该 owner 的文件系统访问权限。
pub fn detect_pending_transactions(
    directory: &Path,
    tag: &str,
    policy: &mut impl RecoveryPolicy,
) -> Result<Vec<PathBuf>, DurableTransactionError> {
    let directory = resolve_recovery_directory(directory)?;
    let _owner = TransactionOwnerLock::acquire(&directory, TransactionPhase::Recovery)?;
    let mode = policy.recovery_mode();
    let pending = load_pending_transactions(&directory, tag, policy, mode)?;
    let mut paths = pending
        .journals
        .into_iter()
        .map(|(path, _, _)| path)
        .chain(pending.atomic_intent_orphans)
        .collect::<Vec<_>>();
    paths.sort();
    Ok(paths)
}

/// 在 owner 锁内完成重启恢复；全部日志先通过领域策略与文件证据校验，之后才允许回滚或清理。
/// 错误会保留未完成证据，调用端应停止该 owner 的新提交并修复恢复条件。
pub fn recover_pending_transactions(
    directory: &Path,
    tag: &str,
    policy: &mut impl RecoveryPolicy,
) -> Result<DurableRecoveryReport, DurableTransactionError> {
    let directory = resolve_recovery_directory(directory)?;
    let _owner = TransactionOwnerLock::acquire(&directory, TransactionPhase::Recovery)?;
    let mode = policy.recovery_mode();
    let pending = load_pending_transactions(&directory, tag, policy, mode)?;
    let orphan_count = pending.atomic_intent_orphans.len();
    for orphan in pending.atomic_intent_orphans {
        remove_reserved_if_exists(&orphan).map_err(|source| operation(&orphan, source))?;
    }
    let mut rollback_count = 0;
    let mut cleanup_count = 0;
    for (path, journal, valid_len) in pending.journals {
        truncate_torn_tail(&path, valid_len)?;
        let rolls_back = mode == RecoveryMode::RestoreOriginal
            && journal.phase == super::schema::JournalPhase::Active;
        recover_journal(&path, &journal, mode)?;
        rollback_count += usize::from(rolls_back);
        cleanup_count += 1;
    }
    Ok(DurableRecoveryReport::new(
        rollback_count,
        cleanup_count,
        orphan_count,
    ))
}

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
