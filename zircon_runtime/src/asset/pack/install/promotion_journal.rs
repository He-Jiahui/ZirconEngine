use std::path::{Path, PathBuf};

use crate::asset::project::ProjectPaths;
use crate::core::resource::io::transaction::{
    recover_pending_transactions, JournalDocument, RecoveryPolicy,
};

use super::ZrPackDeltaInstallError;

pub(super) const TRANSACTION_TAG: &str = "pack";

// 先解析并检查三个文件路径的别名，journal 位置由 installed 文件名固定，恢复时再按请求路径限定写入权限。
pub(super) struct PromotionPaths {
    pub(super) staged: PathBuf,
    pub(super) installed: PathBuf,
    pub(super) backup: Option<PathBuf>,
    pub(super) journal: PathBuf,
}

impl PromotionPaths {
    pub(super) fn new(
        staged: &Path,
        installed: &Path,
        backup: Option<&Path>,
    ) -> Result<Self, ZrPackDeltaInstallError> {
        let resolve = |path: &Path| {
            ProjectPaths::resolve_root(path).map_err(|error| ZrPackDeltaInstallError::ReadFailed {
                path: path.to_path_buf(),
                error: error.to_string(),
            })
        };
        let staged = resolve(staged)?;
        let installed = resolve(installed)?;
        let backup = backup.map(resolve).transpose()?;
        if paths_alias(&staged, &installed)?
            || backup
                .as_ref()
                .map(|path| {
                    Ok::<_, ZrPackDeltaInstallError>(
                        paths_alias(path, &staged)? || paths_alias(path, &installed)?,
                    )
                })
                .transpose()?
                .unwrap_or(false)
        {
            return Err(ZrPackDeltaInstallError::InvalidPromotionPaths(
                "staged, installed and backup packs must have distinct identities".into(),
            ));
        }
        let mut journal_name = installed
            .file_name()
            .ok_or_else(|| {
                ZrPackDeltaInstallError::InvalidPromotionPaths(
                    "installed pack has no filename".into(),
                )
            })?
            .to_os_string();
        journal_name.push(".promotion-journal");
        let journal = installed.with_file_name(journal_name);
        Ok(Self {
            staged,
            installed,
            backup,
            journal,
        })
    }

    pub(super) fn recover(&self) -> Result<(), ZrPackDeltaInstallError> {
        if !self.journal.exists() {
            return Ok(());
        }
        let mut policy = PromotionRecoveryPolicy(self);
        recover_pending_transactions(&self.journal, TRANSACTION_TAG, &mut policy)
            .map_err(|error| self.transaction_error(error))?;
        Ok(())
    }

    pub(super) fn transaction_error(
        &self,
        error: impl std::fmt::Display,
    ) -> ZrPackDeltaInstallError {
        ZrPackDeltaInstallError::TransactionFailed {
            journal_directory: self.journal.clone(),
            error: error.to_string(),
        }
    }
}

pub(super) fn paths_alias(left: &Path, right: &Path) -> Result<bool, ZrPackDeltaInstallError> {
    let error =
        |source: std::io::Error| ZrPackDeltaInstallError::InvalidPromotionPaths(source.to_string());
    if ProjectPaths::same_lexical_path(left, right).map_err(error)? {
        return Ok(true);
    }
    if left.try_exists().map_err(error)? && right.try_exists().map_err(error)? {
        return same_file::is_same_file(left, right).map_err(error);
    }
    Ok(false)
}

struct PromotionRecoveryPolicy<'a>(&'a PromotionPaths);

// installed 目标必须退休本次 staging；backup 目标不得带退休路径，其它 journal 权限一律拒绝。
impl RecoveryPolicy for PromotionRecoveryPolicy<'_> {
    fn validate_document(
        &self,
        _journal_path: &Path,
        document: &JournalDocument,
    ) -> Result<(), String> {
        let paths = self.0;
        if document.target() == paths.installed {
            let mut retired = document.retired_paths();
            if retired.next() == Some(paths.staged.as_path()) && retired.next().is_none() {
                return Ok(());
            }
        } else if paths.backup.as_deref() == Some(document.target())
            && document.retired_paths().next().is_none()
        {
            return Ok(());
        }
        Err(format!(
            "pack promotion journal target {} is outside the requested installation",
            document.target().display()
        ))
    }
}
