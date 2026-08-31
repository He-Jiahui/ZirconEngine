use std::fs;
use std::io;
use std::path::Path;

use super::reader::document_key_from_journal_path;
use super::{
    DurableJournal, DurableJournalError, JournalDiscoveryEntry, JournalDiscoveryIssue,
    JournalDiscoveryReport,
};

impl DurableJournal {
    /// Scans all durable journal directories without creating missing recovery state.
    ///
    /// Corrupt entries are isolated as issues so one damaged document cannot hide another
    /// document's recoverable prefix.
    pub fn discover(&self) -> Result<JournalDiscoveryReport, DurableJournalError> {
        let root = self.journal_root();
        let directory = match fs::read_dir(&root) {
            Ok(directory) => directory,
            Err(source) if source.kind() == io::ErrorKind::NotFound => {
                return Ok(JournalDiscoveryReport::default());
            }
            Err(source) => {
                return Err(DurableJournalError::Io {
                    operation: "enumerate durable journals",
                    path: root,
                    source,
                });
            }
        };

        let mut entries = Vec::new();
        let mut issues = Vec::new();
        for entry in directory {
            let entry = entry.map_err(|source| DurableJournalError::Io {
                operation: "enumerate durable journal directory entry",
                path: root.clone(),
                source,
            })?;
            let directory_path = entry.path();
            let file_type = entry
                .file_type()
                .map_err(|source| DurableJournalError::Io {
                    operation: "inspect durable journal directory entry",
                    path: directory_path.clone(),
                    source,
                })?;
            if !file_type.is_dir() {
                continue;
            }
            self.discover_directory(&directory_path, &mut entries, &mut issues);
        }
        entries.sort_by(|left, right| left.document().cmp(right.document()));
        issues.sort_by(|left, right| left.path().cmp(right.path()));
        Ok(JournalDiscoveryReport::new(entries, issues))
    }

    fn discover_directory(
        &self,
        directory: &Path,
        entries: &mut Vec<JournalDiscoveryEntry>,
        issues: &mut Vec<JournalDiscoveryIssue>,
    ) {
        let journal_path = directory.join("transactions.zjr");
        let document = match document_key_from_journal_path(&journal_path) {
            Ok(document) => document,
            Err(error) => {
                issues.push(JournalDiscoveryIssue::Journal {
                    path: journal_path,
                    error,
                });
                return;
            }
        };
        let actual_key = directory
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_owned();
        if actual_key != document.as_str() {
            issues.push(JournalDiscoveryIssue::DirectoryKeyMismatch {
                path: journal_path,
                expected_key: document.as_str().to_owned(),
                actual_key,
            });
            return;
        }
        match self.read(&document) {
            Ok(report) => entries.push(JournalDiscoveryEntry::new(document, journal_path, report)),
            Err(error) => issues.push(JournalDiscoveryIssue::Journal {
                path: journal_path,
                error,
            }),
        }
    }
}
