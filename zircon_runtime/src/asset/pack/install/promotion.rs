use std::path::Path;

use crate::asset::pack::ZrPackReader;
use crate::core::resource::io::transaction::{
    commit_prepared_files, DurableCommitDisposition, DurableCommitReport, PreparedFileWrite,
    TransactionFault,
};

use super::file_io::{optional_backup_path, read_pack_file};
use super::promotion_journal::PromotionPaths;
use super::{ZrPackDeltaInstallError, ZrPackPromotionMethod, ZrPackPromotionReport};

pub(super) fn promote_staged_pack(
    staged_pack: &Path,
    installed_pack: &Path,
    backup_pack: Option<impl AsRef<Path>>,
) -> Result<ZrPackPromotionReport, ZrPackDeltaInstallError> {
    promote_with_fault(
        staged_pack,
        installed_pack,
        optional_backup_path(backup_pack).as_deref(),
        TransactionFault::None,
    )
}

// 先恢复遗留 journal 并解析 staging；backup、installed 替换及带摘要校验的 staging 退休进入同一 durable transaction。
pub(super) fn promote_with_fault(
    staged_pack: &Path,
    installed_pack: &Path,
    backup_pack: Option<&Path>,
    fault: TransactionFault,
) -> Result<ZrPackPromotionReport, ZrPackDeltaInstallError> {
    let paths = PromotionPaths::new(staged_pack, installed_pack, backup_pack)?;
    paths.recover()?;
    let staged_bytes = read_pack_file(&paths.staged)?;
    let staged_size = staged_bytes.len() as u64;
    let staged_digest = blake3::hash(&staged_bytes).to_hex().to_string();
    let staged_reader = ZrPackReader::from_bytes(staged_bytes)?;
    let installed_manifest = staged_reader.manifest().clone();
    let mut writes = Vec::with_capacity(2);
    if let Some(backup) = &paths.backup {
        let old_bytes = read_pack_file(&paths.installed)?;
        writes.push(PreparedFileWrite::new(backup, old_bytes));
    }
    writes.push(
        PreparedFileWrite::new(&paths.installed, staged_reader.into_bytes())
            .retiring_with_expected_digest(&paths.staged, staged_digest),
    );
    let mut report = DurableCommitReport::default();
    let disposition = commit_prepared_files(
        &paths.journal,
        super::promotion_journal::TRANSACTION_TAG,
        writes,
        fault,
        &mut report,
    )
    .map_err(|error| paths.transaction_error(error))?;
    if disposition == DurableCommitDisposition::CommitRecoveryDeferred {
        return Err(ZrPackDeltaInstallError::RecoveryRequired {
            journal_directory: paths.journal,
        });
    }
    Ok(ZrPackPromotionReport {
        installed_pack: installed_pack.to_path_buf(),
        backup_pack: backup_pack.map(Path::to_path_buf),
        staged_pack: staged_pack.to_path_buf(),
        installed_manifest,
        installed_size: staged_size,
        promotion_method: ZrPackPromotionMethod::AtomicReplacement,
    })
}

#[cfg(test)]
#[path = "tests/promotion_tests.rs"]
mod tests;
