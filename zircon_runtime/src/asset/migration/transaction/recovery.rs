use std::collections::HashSet;
use std::io;
use std::path::{Path, PathBuf};

use super::toml_evidence::TomlEvidenceReader;
use super::{journal_owner, map_transaction_error, recovery_io};
use crate::asset::migration::AssetMigrationError;
use crate::asset::project::ProjectPaths;
use crate::core::resource::io::transaction::{
    JournalDocument, RecoveryPolicy, detect_pending_transactions as detect_core_transactions,
    recover_pending_transactions as recover_core_transactions,
};

const EVIDENCE_READ_BUFFER_BYTES: usize = 64 * 1024;

pub(in crate::asset::migration) fn recover_pending_transactions(
    project_root: &Path,
    roots: &[PathBuf],
    allowed_targets: &[PathBuf],
) -> Result<(), AssetMigrationError> {
    let Some(directory) = journal_owner::existing_journal_directory(project_root)? else {
        return Ok(());
    };
    let mut policy = MigrationRecoveryPolicy::new(roots, allowed_targets)?;
    recover_core_transactions(&directory, "migrate", &mut policy)
        .map(|_| ())
        .map_err(map_transaction_error)
}

pub(in crate::asset::migration) fn detect_pending_transactions(
    project_root: &Path,
    roots: &[PathBuf],
    allowed_targets: &[PathBuf],
) -> Result<Vec<PathBuf>, AssetMigrationError> {
    let Some(directory) = journal_owner::existing_journal_directory(project_root)? else {
        return Ok(Vec::new());
    };
    let mut policy = MigrationRecoveryPolicy::new(roots, allowed_targets)?;
    detect_core_transactions(&directory, "migrate", &mut policy).map_err(map_transaction_error)
}

struct MigrationRecoveryPolicy {
    roots: Vec<PathBuf>,
    allowed_targets: HashSet<String>,
    evidence: TomlEvidenceReader,
}

impl MigrationRecoveryPolicy {
    fn new(roots: &[PathBuf], allowed_targets: &[PathBuf]) -> Result<Self, AssetMigrationError> {
        let roots = roots
            .iter()
            .map(|root| {
                ProjectPaths::resolve_existing_path(root)
                    .map_err(|source| recovery_io(root, source))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let allowed_targets = allowed_targets
            .iter()
            .filter_map(|path| path_identity(path))
            .collect();
        Ok(Self {
            roots,
            allowed_targets,
            evidence: TomlEvidenceReader::new(EVIDENCE_READ_BUFFER_BYTES),
        })
    }
}

impl RecoveryPolicy for MigrationRecoveryPolicy {
    fn validate_document(
        &self,
        _journal_path: &Path,
        document: &JournalDocument,
    ) -> Result<(), String> {
        self.validate_scanned_target(document.target())?;
        for retired in document.retired_paths() {
            self.validate_scanned_target(retired)?;
            validate_retired_pair(document.target(), retired)?;
        }
        Ok(())
    }

    fn digest_file(&mut self, path: &Path) -> io::Result<String> {
        self.evidence.stream_file_digest(path)
    }
}

impl MigrationRecoveryPolicy {
    fn validate_scanned_target(&self, path: &Path) -> Result<(), String> {
        let identity = path_identity(path).ok_or_else(|| {
            format!(
                "migration target has no physical identity: {}",
                path.display()
            )
        })?;
        if !self.allowed_targets.contains(&identity) {
            return Err(format!(
                "migration target was not produced by the scanner: {}",
                path.display()
            ));
        }
        let resolved = resolve_existing_or_parent(path).ok_or_else(|| {
            format!(
                "migration target has no canonical parent: {}",
                path.display()
            )
        })?;
        if !self.roots.iter().any(|root| resolved.starts_with(root)) {
            return Err(format!(
                "migration target escapes project asset roots: {}",
                path.display()
            ));
        }
        Ok(())
    }
}

fn validate_retired_pair(target: &Path, retired: &Path) -> Result<(), String> {
    if target.parent() != retired.parent() {
        return Err("retired path must share the target directory".to_owned());
    }
    let retired_name = retired
        .file_name()
        .and_then(|name| name.to_str())
        .and_then(|name| name.strip_suffix(".meta.toml"))
        .ok_or_else(|| "retired path must end in .meta.toml".to_owned())?;
    let expected = format!("{retired_name}.zmeta");
    if target.file_name().and_then(|name| name.to_str()) != Some(expected.as_str()) {
        return Err("retired path and target do not describe the same sidecar".to_owned());
    }
    Ok(())
}

fn path_identity(path: &Path) -> Option<String> {
    let resolved = resolve_existing_or_parent(path)?;
    #[cfg(windows)]
    return Some(normalize_windows_path_identity(
        resolved.to_string_lossy().into_owned(),
    ));
    #[cfg(not(windows))]
    Some(resolved.to_string_lossy().into_owned())
}

#[cfg(windows)]
fn normalize_windows_path_identity(mut identity: String) -> String {
    identity.make_ascii_lowercase();
    identity
}

#[cfg(all(test, windows))]
#[path = "recovery/path_identity_tests.rs"]
mod path_identity_tests;

fn resolve_existing_or_parent(path: &Path) -> Option<PathBuf> {
    if path.exists() {
        ProjectPaths::resolve_existing_path(path).ok()
    } else {
        let name = path.file_name()?;
        ProjectPaths::resolve_existing_path(path.parent()?)
            .ok()
            .map(|parent| parent.join(name))
    }
}
