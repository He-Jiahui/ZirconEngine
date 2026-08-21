use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use crate::asset::project::ProjectPaths;

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
            path: path.map(ProjectPaths::display_path),
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
            path: ProjectPaths::display_path(path),
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AssetMigrationMetrics {
    pub(super) entry_visits: usize,
    pub(super) directory_reads: usize,
    pub(super) directory_sorts: usize,
    pub(super) resolver_index_lookups: usize,
    pub(super) document_reads: usize,
    pub(super) document_parses: usize,
    pub(super) reference_visits: usize,
    pub(super) output_bytes: usize,
}

impl AssetMigrationMetrics {
    pub fn entry_visits(&self) -> usize {
        self.entry_visits
    }

    pub fn directory_reads(&self) -> usize {
        self.directory_reads
    }

    pub fn directory_sorts(&self) -> usize {
        self.directory_sorts
    }

    pub fn resolver_index_lookups(&self) -> usize {
        self.resolver_index_lookups
    }

    pub fn document_reads(&self) -> usize {
        self.document_reads
    }

    pub fn document_parses(&self) -> usize {
        self.document_parses
    }

    pub fn reference_visits(&self) -> usize {
        self.reference_visits
    }

    pub fn output_bytes(&self) -> usize {
        self.output_bytes
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssetMigrationReport {
    mode: AssetMigrationMode,
    scanned_files: usize,
    changed_files: Vec<AssetMigrationChange>,
    issues: Vec<AssetMigrationIssue>,
    applied: bool,
    pub(super) metrics: AssetMigrationMetrics,
}

impl AssetMigrationReport {
    pub(super) fn new(mode: AssetMigrationMode) -> Self {
        Self {
            mode,
            scanned_files: 0,
            changed_files: Vec::new(),
            issues: Vec::new(),
            applied: false,
            metrics: AssetMigrationMetrics::default(),
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

    pub fn metrics(&self) -> &AssetMigrationMetrics {
        &self.metrics
    }

    pub fn format_text(&self) -> String {
        let mut output = String::new();
        write!(
            output,
            "migrate-assets mode={:?} scanned={} changed={} issues={} applied={}",
            self.mode,
            self.scanned_files,
            self.changed_files.len(),
            self.issues.len(),
            self.applied
        )
        .expect("writing to String cannot fail");
        for change in &self.changed_files {
            write!(
                output,
                "\nchange path={} references={}",
                change.path.display(),
                change.reference_count
            )
            .expect("writing to String cannot fail");
        }
        for issue in &self.issues {
            write!(output, "\nissue kind={:?} path=", issue.kind)
                .expect("writing to String cannot fail");
            if let Some(path) = &issue.path {
                write!(output, "{}", path.display()).expect("writing to String cannot fail");
            } else {
                output.push_str("<project>");
            }
            write!(output, " message={}", issue.message).expect("writing to String cannot fail");
        }
        output
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
