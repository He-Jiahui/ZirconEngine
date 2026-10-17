use super::{
    check_paths, journal, lock_project_generation, pending_path, PluginApplyError,
    PluginApplyErrorKind, ProjectPaths, TAG,
};
use crate::core::resource::io::transaction::{
    recover_pending_transactions, JournalDocument, RecoveryPolicy,
};
use std::{fs, io, path::Path};

/// Data-only probe. It never creates the transaction owner's sibling lock file.
pub fn ensure_plugin_apply_ready(paths: &ProjectPaths) -> Result<(), PluginApplyError> {
    check_paths(paths)?;
    reject_links(&journal(paths))
        .map_err(|e| PluginApplyError::new(PluginApplyErrorKind::Io, e))?;
    let mut entries = match fs::read_dir(journal(paths)) {
        Ok(entries) => entries,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(PluginApplyError::new(PluginApplyErrorKind::Io, e)),
    };
    if let Some(entry) = entries.next() {
        entry.map_err(|e| PluginApplyError::new(PluginApplyErrorKind::Io, e))?;
        return Err(PluginApplyError::new(
            PluginApplyErrorKind::RecoveryRequired,
            "plugin enablement journal must be recovered before manifest parsing",
        ));
    }
    Ok(())
}
pub fn recover_plugin_apply_files(paths: &ProjectPaths) -> Result<(), PluginApplyError> {
    let _generation = lock_project_generation(paths.root())
        .map_err(|e| PluginApplyError::new(PluginApplyErrorKind::Io, e))?;
    recover_plugin_apply_files_locked(paths)
}
/// Caller already owns the project generation lock. No runtime capability is touched.
pub(crate) fn recover_plugin_apply_files_locked(
    paths: &ProjectPaths,
) -> Result<(), PluginApplyError> {
    check_paths(paths)?;
    reject_links(&journal(paths))
        .map_err(|e| PluginApplyError::new(PluginApplyErrorKind::Io, e))?;
    if !journal(paths).exists() {
        return Ok(());
    }
    let mut policy = PluginApplyRecoveryPolicy { paths };
    recover_pending_transactions(&journal(paths), TAG, &mut policy)
        .map_err(|e| PluginApplyError::new(PluginApplyErrorKind::RecoveryRequired, e))?;
    ensure_plugin_apply_ready(paths)
}
struct PluginApplyRecoveryPolicy<'a> {
    paths: &'a ProjectPaths,
}
impl RecoveryPolicy for PluginApplyRecoveryPolicy<'_> {
    fn validate_document(
        &self,
        journal_path: &Path,
        document: &JournalDocument,
    ) -> Result<(), String> {
        check_paths(self.paths).map_err(|e| e.to_string())?;
        if journal_path.parent() != Some(journal(self.paths).as_path())
            || (document.target() != self.paths.manifest_path()
                && document.target() != pending_path(self.paths))
            || document.retired_paths().next().is_some()
        {
            return Err(
                "plugin enablement journal contains an unauthorized document or retirement".into(),
            );
        }
        reject_links(document.target()).map_err(|e| e.to_string())?;
        Ok(())
    }
}
fn reject_links(path: &Path) -> io::Result<()> {
    for entry in path.ancestors() {
        let metadata = match fs::symlink_metadata(entry) {
            Ok(m) => m,
            Err(e) if e.kind() == io::ErrorKind::NotFound => continue,
            Err(e) => return Err(e),
        };
        #[cfg(windows)]
        let linked = {
            use std::os::windows::fs::MetadataExt;
            metadata.file_attributes() & 0x400 != 0
        };
        #[cfg(not(windows))]
        let linked = metadata.file_type().is_symlink();
        if linked {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "linked plugin enablement path",
            ));
        }
        if entry != path && !metadata.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "plugin enablement parent is not a directory",
            ));
        }
    }
    Ok(())
}
