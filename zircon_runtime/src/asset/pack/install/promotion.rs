use std::path::{Path, PathBuf};

#[cfg(test)]
use std::cell::Cell;

use crate::asset::pack::{ZrPackError, ZrPackReader};

use super::{
    ZrPackDeltaInstallError, ZrPackPromotionMethod, ZrPackPromotionReport,
    file_io::{
        copy_pack_file, optional_backup_path, read_pack_file, remove_pack_file, rename_pack_file,
    },
};

pub(super) fn promote_staged_pack(
    staged_pack: &Path,
    installed_pack: &Path,
    backup_pack: Option<impl AsRef<Path>>,
) -> Result<ZrPackPromotionReport, ZrPackDeltaInstallError> {
    promote_staged_pack_with_ops(
        staged_pack,
        installed_pack,
        optional_backup_path(backup_pack),
        &FsPromotionFileOps,
    )
}

#[cfg(test)]
pub(super) fn promote_staged_pack_with_forced_staged_rename_failure(
    staged_pack: &Path,
    installed_pack: &Path,
    backup_pack: Option<impl AsRef<Path>>,
) -> Result<ZrPackPromotionReport, ZrPackDeltaInstallError> {
    let file_ops = ForcedStagedRenameFailureFileOps::new(staged_pack, installed_pack);
    promote_staged_pack_with_ops(
        staged_pack,
        installed_pack,
        optional_backup_path(backup_pack),
        &file_ops,
    )
}

fn promote_staged_pack_with_ops(
    staged_pack: &Path,
    installed_pack: &Path,
    backup_pack: Option<PathBuf>,
    file_ops: &impl PromotionFileOps,
) -> Result<ZrPackPromotionReport, ZrPackDeltaInstallError> {
    let staged_bytes = read_pack_file(staged_pack)?;
    let staged_reader = ZrPackReader::from_bytes(staged_bytes)?;
    let staged_size = u64::try_from(
        std::fs::metadata(staged_pack)
            .map_err(|error| ZrPackDeltaInstallError::ReadFailed {
                path: staged_pack.to_path_buf(),
                error: error.to_string(),
            })?
            .len(),
    )
    .map_err(|_| ZrPackError::SizeOverflow)?;

    if let Some(backup_pack) = backup_pack.as_deref() {
        file_ops.rename(installed_pack, backup_pack)?;
    }
    let promotion_method = promote_staged_to_installed(
        staged_pack,
        installed_pack,
        staged_reader.manifest(),
        staged_size,
        file_ops,
    )
    .map_err(|error| {
        restore_backup_after_failed_promotion(installed_pack, backup_pack.as_deref(), file_ops);
        error
    })?;

    Ok(ZrPackPromotionReport {
        installed_pack: installed_pack.to_path_buf(),
        backup_pack,
        staged_pack: staged_pack.to_path_buf(),
        installed_manifest: staged_reader.manifest().clone(),
        installed_size: staged_size,
        promotion_method,
    })
}

fn promote_staged_to_installed(
    staged_pack: &Path,
    installed_pack: &Path,
    staged_manifest: &crate::asset::pack::ZrPackDocumentManifest,
    staged_size: u64,
    file_ops: &impl PromotionFileOps,
) -> Result<ZrPackPromotionMethod, ZrPackDeltaInstallError> {
    match file_ops.rename(staged_pack, installed_pack) {
        Ok(()) => Ok(ZrPackPromotionMethod::Renamed),
        Err(rename_error) => {
            if installed_pack.exists() {
                return Err(rename_error);
            }
            file_ops.copy(staged_pack, installed_pack)?;
            validate_promoted_pack(installed_pack, staged_manifest, staged_size)?;
            file_ops.remove(staged_pack)?;
            Ok(ZrPackPromotionMethod::CopiedAfterRenameFailure)
        }
    }
}

fn validate_promoted_pack(
    installed_pack: &Path,
    staged_manifest: &crate::asset::pack::ZrPackDocumentManifest,
    staged_size: u64,
) -> Result<(), ZrPackDeltaInstallError> {
    let installed_bytes = read_pack_file(installed_pack)?;
    let installed_reader = ZrPackReader::from_bytes(installed_bytes)?;
    if installed_reader.manifest() != staged_manifest {
        return Err(ZrPackError::DeltaTargetManifestMismatch.into());
    }
    let installed_size = std::fs::metadata(installed_pack)
        .map_err(|error| ZrPackDeltaInstallError::ReadFailed {
            path: installed_pack.to_path_buf(),
            error: error.to_string(),
        })?
        .len();
    if installed_size != staged_size {
        return Err(ZrPackError::DeltaTargetManifestMismatch.into());
    }
    Ok(())
}

fn restore_backup_after_failed_promotion(
    installed_pack: &Path,
    backup_pack: Option<&Path>,
    file_ops: &impl PromotionFileOps,
) {
    let _ = file_ops.remove(installed_pack);
    if let Some(backup_pack) = backup_pack {
        let _ = file_ops.rename(backup_pack, installed_pack);
    }
}

trait PromotionFileOps {
    fn rename(&self, source: &Path, destination: &Path) -> Result<(), ZrPackDeltaInstallError>;
    fn copy(&self, source: &Path, destination: &Path) -> Result<(), ZrPackDeltaInstallError>;
    fn remove(&self, path: &Path) -> Result<(), ZrPackDeltaInstallError>;
}

#[derive(Clone, Copy, Debug, Default)]
struct FsPromotionFileOps;

impl PromotionFileOps for FsPromotionFileOps {
    fn rename(&self, source: &Path, destination: &Path) -> Result<(), ZrPackDeltaInstallError> {
        rename_pack_file(source, destination)
    }

    fn copy(&self, source: &Path, destination: &Path) -> Result<(), ZrPackDeltaInstallError> {
        copy_pack_file(source, destination)
    }

    fn remove(&self, path: &Path) -> Result<(), ZrPackDeltaInstallError> {
        remove_pack_file(path)
    }
}

#[cfg(test)]
#[derive(Debug)]
struct ForcedStagedRenameFailureFileOps {
    staged_pack: PathBuf,
    installed_pack: PathBuf,
    failed_staged_rename: Cell<bool>,
}

#[cfg(test)]
impl ForcedStagedRenameFailureFileOps {
    fn new(staged_pack: &Path, installed_pack: &Path) -> Self {
        Self {
            staged_pack: staged_pack.to_path_buf(),
            installed_pack: installed_pack.to_path_buf(),
            failed_staged_rename: Cell::new(false),
        }
    }
}

#[cfg(test)]
impl PromotionFileOps for ForcedStagedRenameFailureFileOps {
    fn rename(&self, source: &Path, destination: &Path) -> Result<(), ZrPackDeltaInstallError> {
        if !self.failed_staged_rename.get()
            && source == self.staged_pack
            && destination == self.installed_pack
        {
            self.failed_staged_rename.set(true);
            return Err(ZrPackDeltaInstallError::RenameFailed {
                source: source.to_path_buf(),
                destination: destination.to_path_buf(),
                error: "forced staged rename failure".to_string(),
            });
        }
        rename_pack_file(source, destination)
    }

    fn copy(&self, source: &Path, destination: &Path) -> Result<(), ZrPackDeltaInstallError> {
        copy_pack_file(source, destination)
    }

    fn remove(&self, path: &Path) -> Result<(), ZrPackDeltaInstallError> {
        remove_pack_file(path)
    }
}
