use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use zircon_runtime::core::resource::io::{atomic_write, atomic_write_new};
use zircon_runtime_interface::project::ProjectActivationOperationId;

use super::{
    ProjectSessionEffect, ProjectSessionEffectDisposition, ProjectSessionEffectLedger,
    ProjectSessionEffectLedgerError, ProjectSessionEffectLedgerPhase, ProjectSessionRecoveryStatus,
};

const SESSION_EFFECT_LEDGER_DIRECTORY: &str = "session-effects";
const SESSION_EFFECT_LEDGER_EXTENSION: &str = "json";
const MAX_SESSION_EFFECT_LEDGER_BYTES: usize = 8 * 1024;

/// Owns the durable effect record for the full admitted project-session lifecycle.
#[derive(Debug)]
pub(crate) struct ProjectSessionEffectLedgerStore {
    project_root: PathBuf,
    path: PathBuf,
    ledger: ProjectSessionEffectLedger,
}

impl ProjectSessionEffectLedgerStore {
    pub(crate) fn create(
        project_root: &Path,
        operation_id: ProjectActivationOperationId,
    ) -> Result<Self, ProjectSessionEffectLedgerError> {
        let path = Self::path_for_operation(project_root, operation_id);
        let directory = path
            .parent()
            .expect("session effect ledger path always has a parent");
        fs::create_dir_all(directory).map_err(|source| ProjectSessionEffectLedgerError::Io {
            operation: "create session effect ledger directory",
            path: directory.to_path_buf(),
            source,
        })?;
        let ledger = ProjectSessionEffectLedger::for_operation(operation_id);
        let bytes = Self::encode(&path, &ledger)?;
        atomic_write_new(&path, &bytes).map_err(|source| ProjectSessionEffectLedgerError::Io {
            operation: "create session effect ledger",
            path: path.clone(),
            source,
        })?;
        Ok(Self {
            project_root: project_root.to_path_buf(),
            path,
            ledger,
        })
    }

    pub(crate) fn load(
        project_root: &Path,
        operation_id: ProjectActivationOperationId,
    ) -> Result<Self, ProjectSessionEffectLedgerError> {
        let path = Self::path_for_operation(project_root, operation_id);
        let source = Self::read_bounded(&path, "read session effect ledger")?;
        let ledger = Self::decode(&path, &source, operation_id)?;
        Ok(Self {
            project_root: project_root.to_path_buf(),
            path,
            ledger,
        })
    }

    pub(crate) fn inspect_recovery(
        project_root: &Path,
        operation_id: ProjectActivationOperationId,
    ) -> Result<ProjectSessionRecoveryStatus, ProjectSessionEffectLedgerError> {
        let path = Self::path_for_operation(project_root, operation_id);
        let Some(source) =
            Self::read_bounded_if_present(&path, "read session effect ledger for recovery")?
        else {
            return Ok(ProjectSessionRecoveryStatus::Missing);
        };
        Ok(Self::decode(&path, &source, operation_id)?.recovery_status())
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    pub(crate) fn project_root(&self) -> &Path {
        &self.project_root
    }

    pub(crate) fn ledger(&self) -> &ProjectSessionEffectLedger {
        &self.ledger
    }

    pub(crate) fn prepare(
        &mut self,
        effect: ProjectSessionEffect,
    ) -> Result<(), ProjectSessionEffectLedgerError> {
        if matches!(
            self.ledger.disposition(effect),
            Some(
                ProjectSessionEffectDisposition::Prepared
                    | ProjectSessionEffectDisposition::Committed
            )
        ) {
            return Ok(());
        }
        self.mutate(|next| next.prepare(effect))
    }

    pub(crate) fn commit(
        &mut self,
        effect: ProjectSessionEffect,
    ) -> Result<(), ProjectSessionEffectLedgerError> {
        if self.ledger.disposition(effect) == Some(ProjectSessionEffectDisposition::Committed) {
            return Ok(());
        }
        self.mutate(|next| next.commit(effect))
    }

    pub(crate) fn roll_back(
        &mut self,
        effect: ProjectSessionEffect,
    ) -> Result<(), ProjectSessionEffectLedgerError> {
        self.mutate(|next| next.roll_back(effect))
    }

    pub(crate) fn mark_recovery_required(
        &mut self,
        effect: ProjectSessionEffect,
    ) -> Result<(), ProjectSessionEffectLedgerError> {
        if self.ledger.phase() == ProjectSessionEffectLedgerPhase::RecoveryRequired
            && self.ledger.disposition(effect)
                == Some(ProjectSessionEffectDisposition::RecoveryRequired)
        {
            return Ok(());
        }
        self.mutate(|next| next.mark_recovery_required(effect))
    }

    pub(crate) fn roll_back_active_effects(
        &mut self,
    ) -> Result<(), ProjectSessionEffectLedgerError> {
        self.mutate(ProjectSessionEffectLedger::roll_back_active_effects)
    }

    pub(crate) fn require_recovery_for_active_effects(
        &mut self,
    ) -> Result<(), ProjectSessionEffectLedgerError> {
        self.mutate(ProjectSessionEffectLedger::require_recovery_for_active_effects)
    }

    pub(crate) fn begin_ready(&mut self) -> Result<(), ProjectSessionEffectLedgerError> {
        if self.ledger.phase() == ProjectSessionEffectLedgerPhase::Ready {
            return Ok(());
        }
        self.mutate(ProjectSessionEffectLedger::begin_ready)
    }

    pub(crate) fn finish_aborted_activation(
        &mut self,
    ) -> Result<(), ProjectSessionEffectLedgerError> {
        if self.ledger.phase() == ProjectSessionEffectLedgerPhase::Closed {
            return Ok(());
        }
        self.mutate(ProjectSessionEffectLedger::finish_aborted_activation)
    }

    pub(crate) fn begin_closing(&mut self) -> Result<(), ProjectSessionEffectLedgerError> {
        if self.ledger.phase() == ProjectSessionEffectLedgerPhase::Closing {
            return Ok(());
        }
        self.mutate(ProjectSessionEffectLedger::begin_closing)
    }

    pub(crate) fn finish_closed(&mut self) -> Result<(), ProjectSessionEffectLedgerError> {
        if self.ledger.phase() == ProjectSessionEffectLedgerPhase::Closed {
            return Ok(());
        }
        self.mutate(ProjectSessionEffectLedger::finish_closed)
    }

    /// Removes only a fully closed record. Ready records remain durable for crash arbitration.
    pub(crate) fn cleanup_if_closed(&self) -> Result<bool, ProjectSessionEffectLedgerError> {
        if self.ledger.phase() != ProjectSessionEffectLedgerPhase::Closed {
            return Ok(false);
        }
        match fs::remove_file(&self.path) {
            Ok(()) => Ok(true),
            Err(source) if source.kind() == std::io::ErrorKind::NotFound => Ok(true),
            Err(source) => Err(ProjectSessionEffectLedgerError::Io {
                operation: "remove closed session effect ledger",
                path: self.path.clone(),
                source,
            }),
        }
    }

    fn path_for_operation(
        project_root: &Path,
        operation_id: ProjectActivationOperationId,
    ) -> PathBuf {
        project_root
            .join(".zircon")
            .join(SESSION_EFFECT_LEDGER_DIRECTORY)
            .join(format!(
                "{}.{}",
                operation_id.nonce(),
                SESSION_EFFECT_LEDGER_EXTENSION
            ))
    }

    fn mutate(
        &mut self,
        apply: impl FnOnce(
            &mut ProjectSessionEffectLedger,
        ) -> Result<(), ProjectSessionEffectLedgerError>,
    ) -> Result<(), ProjectSessionEffectLedgerError> {
        let mut next = self.ledger.clone();
        apply(&mut next)?;
        let bytes = Self::encode(&self.path, &next)?;
        atomic_write(&self.path, &bytes).map_err(|source| ProjectSessionEffectLedgerError::Io {
            operation: "persist session effect ledger",
            path: self.path.clone(),
            source,
        })?;
        self.ledger = next;
        Ok(())
    }

    fn encode(
        path: &Path,
        ledger: &ProjectSessionEffectLedger,
    ) -> Result<Vec<u8>, ProjectSessionEffectLedgerError> {
        ledger.validate_persisted_state().map_err(|message| {
            ProjectSessionEffectLedgerError::InvalidRecord {
                path: path.to_path_buf(),
                message,
            }
        })?;
        let bytes = serde_json::to_vec_pretty(ledger).map_err(|source| {
            ProjectSessionEffectLedgerError::Encode {
                path: path.to_path_buf(),
                source,
            }
        })?;
        if bytes.len() > MAX_SESSION_EFFECT_LEDGER_BYTES {
            return Err(ProjectSessionEffectLedgerError::RecordTooLarge {
                path: path.to_path_buf(),
                actual_bytes: bytes.len(),
                max_bytes: MAX_SESSION_EFFECT_LEDGER_BYTES,
            });
        }
        Ok(bytes)
    }

    fn read_bounded(
        path: &Path,
        operation: &'static str,
    ) -> Result<Vec<u8>, ProjectSessionEffectLedgerError> {
        let file = fs::File::open(path).map_err(|source| ProjectSessionEffectLedgerError::Io {
            operation,
            path: path.to_path_buf(),
            source,
        })?;
        Self::read_open_file_bounded(path, operation, file)
    }

    fn read_bounded_if_present(
        path: &Path,
        operation: &'static str,
    ) -> Result<Option<Vec<u8>>, ProjectSessionEffectLedgerError> {
        let file = match fs::File::open(path) {
            Ok(file) => file,
            Err(source) if source.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(source) => {
                return Err(ProjectSessionEffectLedgerError::Io {
                    operation,
                    path: path.to_path_buf(),
                    source,
                });
            }
        };
        Self::read_open_file_bounded(path, operation, file).map(Some)
    }

    fn read_open_file_bounded(
        path: &Path,
        operation: &'static str,
        file: fs::File,
    ) -> Result<Vec<u8>, ProjectSessionEffectLedgerError> {
        let source = read_capped_ledger_bytes(file).map_err(|source| {
            ProjectSessionEffectLedgerError::Io {
                operation,
                path: path.to_path_buf(),
                source,
            }
        })?;
        if source.len() > MAX_SESSION_EFFECT_LEDGER_BYTES {
            return Err(ProjectSessionEffectLedgerError::RecordTooLarge {
                path: path.to_path_buf(),
                actual_bytes: source.len(),
                max_bytes: MAX_SESSION_EFFECT_LEDGER_BYTES,
            });
        }
        Ok(source)
    }

    fn decode(
        path: &Path,
        source: &[u8],
        operation_id: ProjectActivationOperationId,
    ) -> Result<ProjectSessionEffectLedger, ProjectSessionEffectLedgerError> {
        if source.len() > MAX_SESSION_EFFECT_LEDGER_BYTES {
            return Err(ProjectSessionEffectLedgerError::RecordTooLarge {
                path: path.to_path_buf(),
                actual_bytes: source.len(),
                max_bytes: MAX_SESSION_EFFECT_LEDGER_BYTES,
            });
        }
        let ledger =
            serde_json::from_slice::<ProjectSessionEffectLedger>(source).map_err(|source| {
                ProjectSessionEffectLedgerError::InvalidRecord {
                    path: path.to_path_buf(),
                    message: source.to_string(),
                }
            })?;
        if ledger.schema_version() != ProjectSessionEffectLedger::SCHEMA_VERSION {
            return Err(ProjectSessionEffectLedgerError::UnsupportedSchemaVersion {
                path: path.to_path_buf(),
                actual: ledger.schema_version(),
            });
        }
        if ledger.operation_id() != operation_id {
            return Err(ProjectSessionEffectLedgerError::OperationMismatch {
                path: path.to_path_buf(),
            });
        }
        ledger.validate_persisted_state().map_err(|message| {
            ProjectSessionEffectLedgerError::InvalidRecord {
                path: path.to_path_buf(),
                message,
            }
        })?;
        Ok(ledger)
    }
}

fn read_capped_ledger_bytes(mut reader: impl Read) -> std::io::Result<Vec<u8>> {
    let mut source = Vec::with_capacity(MAX_SESSION_EFFECT_LEDGER_BYTES + 1);
    reader
        .by_ref()
        .take((MAX_SESSION_EFFECT_LEDGER_BYTES + 1) as u64)
        .read_to_end(&mut source)?;
    Ok(source)
}

#[cfg(test)]
mod bounded_read_tests {
    use std::io::Cursor;
    use std::path::Path;

    use zircon_runtime_interface::project::{
        ProjectActivationOperationIdGenerator, ProjectLaunchInstanceId,
    };

    use super::{
        read_capped_ledger_bytes, ProjectSessionEffectLedger, ProjectSessionEffectLedgerStore,
        MAX_SESSION_EFFECT_LEDGER_BYTES,
    };

    #[test]
    fn session_effect_ledger_reads_are_capped_before_deserialization() {
        let oversized = vec![b'x'; MAX_SESSION_EFFECT_LEDGER_BYTES * 4];
        let source = read_capped_ledger_bytes(Cursor::new(oversized)).expect("capped read");

        assert_eq!(source.len(), MAX_SESSION_EFFECT_LEDGER_BYTES + 1);
    }

    #[test]
    fn decode_rejects_an_unreachable_closed_effect_inventory() {
        let operation_id =
            ProjectActivationOperationIdGenerator::new(ProjectLaunchInstanceId::new())
                .allocate()
                .expect("fixture operation id");
        let ledger = ProjectSessionEffectLedger::for_operation(operation_id);
        let mut record = serde_json::to_value(ledger).expect("serialize fixture ledger");
        record["phase"] = serde_json::json!("closed");
        record["effects"] = serde_json::json!({ "runtime": "prepared" });
        let source = serde_json::to_vec(&record).expect("encode forged ledger");

        assert!(ProjectSessionEffectLedgerStore::decode(
            Path::new("forged-session-effect-ledger.json"),
            &source,
            operation_id,
        )
        .is_err());
    }
}
