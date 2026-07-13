use std::path::{Path, PathBuf};

use super::AssetMigrationMode;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssetMigrationIssueKind {
    PendingRecovery,
    DanglingReference,
    MissingGuid,
    MissingPath,
    RegistryConflict,
    AmbiguousPath,
    UnsupportedScheme,
    InvalidDocument,
    UnsafePath,
    PathIo,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssetMigrationIssue {
    kind: AssetMigrationIssueKind,
    path: Option<PathBuf>,
    message: String,
}

impl AssetMigrationIssue {
    pub(super) fn new(
        kind: AssetMigrationIssueKind,
        path: Option<PathBuf>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            path,
            message: message.into(),
        }
    }

    pub fn kind(&self) -> AssetMigrationIssueKind {
        self.kind
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssetMigrationChange {
    path: PathBuf,
    reference_count: usize,
}

impl AssetMigrationChange {
    pub(super) fn new(path: PathBuf, reference_count: usize) -> Self {
        Self {
            path,
            reference_count,
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn reference_count(&self) -> usize {
        self.reference_count
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssetMigrationReport {
    mode: AssetMigrationMode,
    scanned_files: usize,
    changed_files: Vec<AssetMigrationChange>,
    issues: Vec<AssetMigrationIssue>,
    applied: bool,
}

impl AssetMigrationReport {
    pub(super) fn new(mode: AssetMigrationMode) -> Self {
        Self {
            mode,
            scanned_files: 0,
            changed_files: Vec::new(),
            issues: Vec::new(),
            applied: false,
        }
    }

    pub fn mode(&self) -> AssetMigrationMode {
        self.mode
    }

    pub fn scanned_files(&self) -> usize {
        self.scanned_files
    }

    pub fn changed_files(&self) -> &[AssetMigrationChange] {
        &self.changed_files
    }

    pub fn issues(&self) -> &[AssetMigrationIssue] {
        &self.issues
    }

    pub fn applied(&self) -> bool {
        self.applied
    }

    pub fn succeeded(&self) -> bool {
        self.issues.is_empty()
    }

    pub fn format_text(&self) -> String {
        let mut lines = vec![format!(
            "migrate-assets mode={:?} scanned={} changed={} issues={} applied={}",
            self.mode,
            self.scanned_files,
            self.changed_files.len(),
            self.issues.len(),
            self.applied
        )];
        lines.extend(self.changed_files.iter().map(|change| {
            format!(
                "change path={} references={}",
                change.path.display(),
                change.reference_count
            )
        }));
        lines.extend(self.issues.iter().map(|issue| {
            format!(
                "issue kind={:?} path={} message={}",
                issue.kind,
                issue.path.as_deref().map_or_else(
                    || "<project>".to_string(),
                    |path| path.display().to_string()
                ),
                issue.message
            )
        }));
        lines.join("\n")
    }

    pub(super) fn set_scanned_files(&mut self, scanned_files: usize) {
        self.scanned_files = scanned_files;
    }

    pub(super) fn push_change(&mut self, change: AssetMigrationChange) {
        self.changed_files.push(change);
    }

    pub(super) fn push_issue(&mut self, issue: AssetMigrationIssue) {
        self.issues.push(issue);
    }

    pub(super) fn mark_applied(&mut self) {
        self.applied = true;
    }
}
