use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use zircon_runtime::asset::project::ProjectPaths;
use zircon_runtime::core::resource::io::atomic_write_new;

use super::{AutosaveDocumentId, AutosaveDocumentOutcome};

const AUTOSAVE_DIRECTORY: &str = "autosave";
const DIAGNOSTICS_DIRECTORY: &str = "diagnostics";
const DIAGNOSTIC_FILE_PREFIX: &str = "outcome-";
const DIAGNOSTIC_FILE_SUFFIX: &str = ".json";
const AUTOSAVE_DIAGNOSTIC_VERSION: u32 = 1;
const MAX_AUTOSAVE_DIAGNOSTIC_RECORDS: usize = 128;
static NEXT_AUTOSAVE_DIAGNOSTIC_ID: AtomicU64 = AtomicU64::new(1);

/// Project-local, append-only terminal autosave evidence.
///
/// Worker jobs publish their own terminal records. The lifecycle service only
/// uses this store for cancellation results synthesized before a worker starts,
/// so normal autosave polling never performs diagnostic file I/O on the UI
/// thread.
#[derive(Clone, Debug)]
pub struct AutosaveDiagnosticStore {
    autosave_root: PathBuf,
}

impl AutosaveDiagnosticStore {
    pub fn new(project_root: impl Into<PathBuf>) -> Self {
        let project_root = project_root.into();
        let project_root = ProjectPaths::resolve_path(&project_root)
            .map(|root| root.into_operation_path())
            .unwrap_or(project_root);
        Self {
            autosave_root: project_root.join(".zircon").join(AUTOSAVE_DIRECTORY),
        }
    }

    pub(crate) fn from_autosave_root(autosave_root: impl Into<PathBuf>) -> Self {
        Self {
            autosave_root: autosave_root.into(),
        }
    }

    pub fn diagnostics_root(&self) -> PathBuf {
        self.autosave_root.join(DIAGNOSTICS_DIRECTORY)
    }

    /// Returns the document's autosave directory for an explicit open-folder action.
    pub fn document_folder(&self, document: &AutosaveDocumentId) -> PathBuf {
        self.autosave_root.join(document.as_str())
    }

    pub fn append(
        &self,
        outcome: &AutosaveDocumentOutcome,
    ) -> Result<AutosaveDiagnosticRecord, AutosaveDiagnosticError> {
        let root = self.diagnostics_root();
        fs::create_dir_all(&root).map_err(|source| AutosaveDiagnosticError::Io {
            operation: "create autosave diagnostics directory",
            path: root.clone(),
            source,
        })?;

        let record = AutosaveDiagnosticRecord::new(next_recorded_at_unix_millis(), outcome.clone());
        let file_name =
            diagnostic_file_name(record.recorded_at_unix_millis(), next_diagnostic_id());
        let path = root.join(file_name);
        let bytes = serde_json::to_vec(&record).map_err(|error| {
            AutosaveDiagnosticError::InvalidRecord {
                path: path.clone(),
                message: error.to_string(),
            }
        })?;
        atomic_write_new(&path, &bytes).map_err(|source| AutosaveDiagnosticError::Io {
            operation: "publish autosave diagnostic",
            path: path.clone(),
            source,
        })?;
        self.rotate_after_append(&path)?;
        Ok(record)
    }

    /// Enumerates valid records without allowing one damaged diagnostic to hide
    /// the remaining project evidence from Welcome or Hub.
    pub fn load(&self) -> Result<AutosaveDiagnosticReport, AutosaveDiagnosticError> {
        let root = self.diagnostics_root();
        let entries = match fs::read_dir(&root) {
            Ok(entries) => entries,
            Err(source) if source.kind() == io::ErrorKind::NotFound => {
                return Ok(AutosaveDiagnosticReport::default());
            }
            Err(source) => {
                return Err(AutosaveDiagnosticError::Io {
                    operation: "read autosave diagnostics directory",
                    path: root,
                    source,
                });
            }
        };

        let mut records = Vec::new();
        let mut diagnostics = Vec::new();
        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(source) => {
                    diagnostics.push(AutosaveDiagnosticReadIssue::new(
                        root.clone(),
                        source.to_string(),
                    ));
                    continue;
                }
            };
            let path = entry.path();
            if !is_diagnostic_path(&path) {
                continue;
            }
            let bytes = match fs::read(&path) {
                Ok(bytes) => bytes,
                Err(source) => {
                    diagnostics.push(AutosaveDiagnosticReadIssue::new(path, source.to_string()));
                    continue;
                }
            };
            match AutosaveDiagnosticRecord::decode(&path, &bytes) {
                Ok(record) => records.push(record),
                Err(error) => diagnostics.push(AutosaveDiagnosticReadIssue::new(path, error)),
            }
        }
        records.sort_by_key(AutosaveDiagnosticRecord::recorded_at_unix_millis);
        Ok(AutosaveDiagnosticReport {
            records,
            diagnostics,
        })
    }

    fn rotate_after_append(&self, persisted: &Path) -> Result<(), AutosaveDiagnosticError> {
        let root = self.diagnostics_root();
        let mut records = fs::read_dir(&root)
            .map_err(|source| AutosaveDiagnosticError::Io {
                operation: "read autosave diagnostics directory for retention",
                path: root.clone(),
                source,
            })?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| is_diagnostic_path(path))
            .collect::<Vec<_>>();
        records.sort();
        while records.len() > MAX_AUTOSAVE_DIAGNOSTIC_RECORDS {
            let oldest = records.remove(0);
            fs::remove_file(&oldest).map_err(|source| {
                AutosaveDiagnosticError::RetentionAfterAppend {
                    persisted: persisted.to_path_buf(),
                    path: oldest,
                    source,
                }
            })?;
        }
        Ok(())
    }
}

/// One durable terminal result. The path values remain project-relative and
/// never grant an authority to overwrite the source document.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutosaveDiagnosticRecord {
    version: u32,
    recorded_at_unix_millis: u64,
    outcome: AutosaveDocumentOutcome,
}

impl AutosaveDiagnosticRecord {
    fn new(recorded_at_unix_millis: u64, outcome: AutosaveDocumentOutcome) -> Self {
        Self {
            version: AUTOSAVE_DIAGNOSTIC_VERSION,
            recorded_at_unix_millis,
            outcome,
        }
    }

    pub const fn recorded_at_unix_millis(&self) -> u64 {
        self.recorded_at_unix_millis
    }

    pub fn outcome(&self) -> &AutosaveDocumentOutcome {
        &self.outcome
    }

    fn decode(path: &Path, bytes: &[u8]) -> Result<Self, String> {
        let record = serde_json::from_slice::<Self>(bytes).map_err(|error| error.to_string())?;
        if record.version != AUTOSAVE_DIAGNOSTIC_VERSION {
            return Err(format!(
                "unsupported autosave diagnostic version {} at {}",
                record.version,
                path.display()
            ));
        }
        Ok(record)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AutosaveDiagnosticReport {
    records: Vec<AutosaveDiagnosticRecord>,
    diagnostics: Vec<AutosaveDiagnosticReadIssue>,
}

impl AutosaveDiagnosticReport {
    pub fn records(&self) -> &[AutosaveDiagnosticRecord] {
        &self.records
    }

    pub fn diagnostics(&self) -> &[AutosaveDiagnosticReadIssue] {
        &self.diagnostics
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AutosaveDiagnosticReadIssue {
    path: PathBuf,
    message: String,
}

impl AutosaveDiagnosticReadIssue {
    fn new(path: impl Into<PathBuf>, message: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            message: message.into(),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

#[derive(Debug, Error)]
pub enum AutosaveDiagnosticError {
    #[error("autosave diagnostic record `{path}` is invalid: {message}")]
    InvalidRecord { path: PathBuf, message: String },
    #[error(
        "autosave diagnostic `{persisted}` was published, but retention failed at `{path}`: {source}"
    )]
    RetentionAfterAppend {
        persisted: PathBuf,
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to {operation} `{path}`: {source}")]
    Io {
        operation: &'static str,
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}

fn diagnostic_file_name(recorded_at_unix_millis: u64, id: u64) -> String {
    format!(
        "{DIAGNOSTIC_FILE_PREFIX}{recorded_at_unix_millis:020}-{id:020}{DIAGNOSTIC_FILE_SUFFIX}"
    )
}

fn is_diagnostic_path(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            name.starts_with(DIAGNOSTIC_FILE_PREFIX) && name.ends_with(DIAGNOSTIC_FILE_SUFFIX)
        })
}

fn next_recorded_at_unix_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| u64::try_from(duration.as_millis()).unwrap_or(u64::MAX))
        .unwrap_or_default()
}

fn next_diagnostic_id() -> u64 {
    NEXT_AUTOSAVE_DIAGNOSTIC_ID.fetch_add(1, Ordering::AcqRel)
}
