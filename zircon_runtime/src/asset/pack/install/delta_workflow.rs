use std::path::Path;

use crate::asset::pack::{ZrPackDeltaReader, ZrPackReader};

use super::{
    file_io::read_pack_file, promotion, promotion_journal::PromotionPaths, staging,
    ZrPackDeltaInstallError, ZrPackDeltaInstallReport, ZrPackPromotionMethod,
    ZrPackPromotionReport,
};

pub(super) fn install_delta(
    base_pack: &Path,
    delta_pack: &Path,
    staged_pack: &Path,
    installed_pack: &Path,
    backup_pack: Option<&Path>,
) -> Result<(ZrPackDeltaInstallReport, ZrPackPromotionReport), ZrPackDeltaInstallError> {
    let paths = PromotionPaths::new(staged_pack, installed_pack, backup_pack)?;
    for input in [base_pack, delta_pack] {
        if super::promotion_journal::paths_alias(input, &paths.staged)?
            || paths
                .backup
                .as_ref()
                .map(|backup| super::promotion_journal::paths_alias(input, backup))
                .transpose()?
                .unwrap_or(false)
        {
            return Err(ZrPackDeltaInstallError::InvalidPromotionPaths(
                "delta inputs must not alias staging or backup destinations".into(),
            ));
        }
    }
    if super::promotion_journal::paths_alias(delta_pack, &paths.installed)? {
        return Err(ZrPackDeltaInstallError::InvalidPromotionPaths(
            "delta input must not alias installed destination".into(),
        ));
    }
    paths.recover()?;
    // A process can stop after pack commit and before receipt or plugin reload. In that case
    // verify the installed target and resume without trying the delta against its new base.
    if paths
        .installed
        .try_exists()
        .map_err(|error| paths.transaction_error(error))?
    {
        let delta = ZrPackDeltaReader::from_bytes(read_pack_file(delta_pack)?)?;
        let installed_bytes = read_pack_file(&paths.installed)?;
        let installed_size = installed_bytes.len() as u64;
        let installed = ZrPackReader::from_bytes(installed_bytes)?;
        if installed.manifest() == &delta.manifest().target {
            if let Some(backup) = paths.backup.as_ref() {
                let backup_reader =
                    ZrPackReader::from_bytes(read_pack_file(backup)?).map_err(|error| {
                        ZrPackDeltaInstallError::BackupPackMismatch {
                            path: backup.clone(),
                            error: error.to_string(),
                        }
                    })?;
                if backup_reader.manifest() != &delta.manifest().base {
                    return Err(ZrPackDeltaInstallError::BackupPackMismatch {
                        path: backup.clone(),
                        error: "its manifest is not the delta base manifest".to_string(),
                    });
                }
            }
            let manifest = installed.manifest().clone();
            return Ok((
                ZrPackDeltaInstallReport {
                    base_pack: base_pack.to_path_buf(),
                    delta_pack: delta_pack.to_path_buf(),
                    staged_pack: staged_pack.to_path_buf(),
                    target_manifest: manifest.clone(),
                    staged_size: installed_size,
                    delta_apply_verified: true,
                },
                ZrPackPromotionReport {
                    installed_pack: installed_pack.to_path_buf(),
                    backup_pack: backup_pack.map(Path::to_path_buf),
                    staged_pack: staged_pack.to_path_buf(),
                    installed_manifest: manifest,
                    installed_size,
                    promotion_method: ZrPackPromotionMethod::AlreadyInstalled,
                },
            ));
        }
    }
    let staging = staging::rebuild_to_staging(base_pack, delta_pack, staged_pack)?;
    let promotion = promotion::promote_staged_pack(staged_pack, installed_pack, backup_pack)?;
    Ok((staging, promotion))
}
