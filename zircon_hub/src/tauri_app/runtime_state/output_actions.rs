use std::{
    fs,
    path::{Component, Path, PathBuf},
};

#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;

use crate::error::HubError;
use crate::process::{open_folder, OpenFolderCommand};
use crate::projects::project_paths_match;
use crate::state::{
    DeliveryMessageId, HubActionKind, HubActionRecord, HubActionStatus, HubMessage, HubMessageId,
    ShellMessageId, TaskOperationKind, TaskStatus,
};
use crate::tauri_app::action_request::{OpenOutputFolderCapability, OpenOutputFolderPayload};

use super::HubRuntimeSession;

impl HubRuntimeSession {
    pub(super) fn open_output_folder(
        &mut self,
        payload: Option<OpenOutputFolderPayload>,
    ) -> Result<(), HubError> {
        let output_dir = match self.resolve_output_folder(payload) {
            Ok(output_dir) => output_dir,
            Err(error) => {
                let (detail, recovery) = error.into_status_messages();
                self.record_output_folder_failure(
                    "Output Folder".to_string(),
                    detail,
                    recovery.unwrap_or_else(|| {
                        HubMessage::new(HubMessageId::Delivery(
                            DeliveryMessageId::ChooseRecordedOutputRecovery,
                        ))
                    }),
                )?;
                return Ok(());
            }
        };
        if !output_dir.is_dir() {
            self.record_output_folder_failure(
                output_dir.to_string_lossy().into_owned(),
                HubMessage::with_params(
                    HubMessageId::Delivery(DeliveryMessageId::OutputFolderDoesNotExist),
                    [output_dir.to_string_lossy().into_owned()],
                ),
                HubMessage::new(HubMessageId::Delivery(
                    DeliveryMessageId::RunWorkflowAgainRecovery,
                )),
            )?;
            return Ok(());
        }
        // Re-open the admitted directory without following a reparse point as
        // close as possible to the shell handoff.  The first admission check
        // protects resolution; this second check narrows the replacement race
        // between resolution and launching Explorer.
        if ensure_no_reparse_directory(&output_dir).is_err() {
            let (detail, recovery) = output_folder_not_recorded(&output_dir).into_status_messages();
            self.record_output_folder_failure(
                output_dir.to_string_lossy().into_owned(),
                detail,
                recovery.unwrap_or_else(|| {
                    HubMessage::new(HubMessageId::Delivery(
                        DeliveryMessageId::ChooseRecordedOutputRecovery,
                    ))
                }),
            )?;
            return Ok(());
        }

        let command = OpenFolderCommand::new(output_dir.clone());
        let command_line = command.command_line();
        match open_folder(&command) {
            Ok(child) => {
                let process_id = child.id();
                crate::state::push_action_record(
                    &mut self.config.action_history,
                    HubActionRecord {
                        finished_unix_ms: crate::projects::now_unix_ms(),
                        action: HubActionKind::OpenOutput,
                        status: HubActionStatus::Success,
                        target: output_dir.to_string_lossy().into_owned(),
                        detail: HubMessage::with_params(
                            HubMessageId::Shell(ShellMessageId::OpenedPath),
                            [output_dir.to_string_lossy().into_owned()],
                        ),
                        log_excerpt: HubMessage::empty(),
                        recovery: None,
                        process_id: Some(process_id),
                        command_line,
                        output_dir: Some(output_dir.clone()),
                    },
                );
                self.task_status = TaskStatus::success(
                    "Output folder opened",
                    HubMessage::raw_text(output_dir.to_string_lossy().into_owned()),
                )
                .with_operation(TaskOperationKind::Process, output_dir.to_string_lossy());
                self.persist()
            }
            Err(error) => self.record_output_folder_failure(
                output_dir.to_string_lossy().into_owned(),
                HubMessage::raw_text(error.to_string()),
                HubMessage::new(HubMessageId::Delivery(
                    DeliveryMessageId::OpenFolderManuallyRecovery,
                )),
            ),
        }
    }

    fn resolve_output_folder(
        &self,
        payload: Option<OpenOutputFolderPayload>,
    ) -> Result<PathBuf, HubError> {
        let Some(payload) = payload else {
            return Err(HubError::status(
                HubMessage::new(HubMessageId::Delivery(
                    DeliveryMessageId::OpenOutputTargetRequired,
                )),
                Some(HubMessage::new(HubMessageId::Delivery(
                    DeliveryMessageId::ChooseRecordedOutputRecovery,
                ))),
            ));
        };

        if let Some(receipt_id) = payload
            .receipt_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            return self.resolve_recorded_history_output(receipt_id);
        }
        if let Some(capability) = payload.capability {
            return self.resolve_output_capability(capability, payload.engine_id.as_deref());
        }

        Err(HubError::status(
            HubMessage::new(HubMessageId::Delivery(
                DeliveryMessageId::OpenOutputTargetRequired,
            )),
            Some(HubMessage::new(HubMessageId::Delivery(
                DeliveryMessageId::ChooseRecordedOutputRecovery,
            ))),
        ))
    }

    fn resolve_recorded_history_output(&self, history_id: &str) -> Result<PathBuf, HubError> {
        let Some(record) = self.config.action_history.iter().find(|record| {
            record.status == HubActionStatus::Success && action_history_id(record) == history_id
        }) else {
            return self.resolve_source_build_receipt(history_id);
        };
        let Some(output_dir) = record.output_dir.as_ref() else {
            return Err(output_folder_not_recorded(Path::new(history_id)));
        };
        self.resolve_recorded_output_path(output_dir)
    }

    fn resolve_source_build_receipt(&self, receipt_id: &str) -> Result<PathBuf, HubError> {
        let Some(rest) = receipt_id.strip_prefix("source-build:") else {
            return Err(output_folder_not_recorded(Path::new(receipt_id)));
        };
        let mut parts = rest.rsplitn(3, ':');
        let Some(index) = parts.next().and_then(|value| value.parse::<usize>().ok()) else {
            return Err(output_folder_not_recorded(Path::new(receipt_id)));
        };
        let Some(finished_unix_ms) = parts.next().and_then(|value| value.parse::<u64>().ok())
        else {
            return Err(output_folder_not_recorded(Path::new(receipt_id)));
        };
        let Some(engine_id) = parts.next().filter(|value| !value.is_empty()) else {
            return Err(output_folder_not_recorded(Path::new(receipt_id)));
        };
        let Some(engine) = self
            .config
            .engines
            .iter()
            .find(|engine| engine.id == engine_id)
        else {
            return Err(output_folder_not_recorded(Path::new(receipt_id)));
        };
        let Some(record) = engine.build_history.get(index) else {
            return Err(output_folder_not_recorded(Path::new(receipt_id)));
        };
        if record.finished_unix_ms != finished_unix_ms
            || !record.status.eq_ignore_ascii_case("success")
        {
            return Err(output_folder_not_recorded(Path::new(receipt_id)));
        }
        // A successful source build record is itself a Hub-issued receipt. It
        // may point at a user-selected output root that is intentionally not a
        // reusable default capability, so validate the exact receipt path
        // without promoting its parent into the general root set.
        self.resolve_generated_output_path(&record.output_dir)
    }

    fn resolve_output_capability(
        &self,
        capability: OpenOutputFolderCapability,
        engine_id: Option<&str>,
    ) -> Result<PathBuf, HubError> {
        match capability {
            OpenOutputFolderCapability::DefaultBuildOutput => {
                if !self
                    .config
                    .settings
                    .default_build_output_grants_open_capability()
                {
                    return Err(output_folder_not_recorded(
                        &self.config.settings.default_build_output_dir,
                    ));
                }
                self.resolve_recorded_output_path(&self.config.settings.default_build_output_dir)
            }
            OpenOutputFolderCapability::DefaultDeviceInstall => {
                if !self
                    .config
                    .settings
                    .default_device_install_grants_open_capability()
                {
                    return Err(output_folder_not_recorded(
                        &self.config.settings.default_device_install_dir,
                    ));
                }
                self.resolve_recorded_output_path(&self.config.settings.default_device_install_dir)
            }
            OpenOutputFolderCapability::SourceEngineOutput => {
                let Some(engine_id) = engine_id.map(str::trim).filter(|value| !value.is_empty())
                else {
                    return Err(output_folder_not_recorded(Path::new(
                        "source-engine-output",
                    )));
                };
                let Some(engine) = self
                    .config
                    .engines
                    .iter()
                    .find(|engine| engine.id == engine_id)
                else {
                    return Err(output_folder_not_recorded(Path::new(engine_id)));
                };
                let has_generated_output = engine.build_history.iter().any(|record| {
                    record.status.eq_ignore_ascii_case("success")
                        && project_paths_match(&record.output_dir, &engine.output_dir)
                });
                if has_generated_output {
                    self.resolve_generated_output_path(&engine.output_dir)
                } else if self
                    .config
                    .settings
                    .default_build_output_grants_open_capability()
                    && project_paths_match(
                        &self.config.settings.default_build_output_dir,
                        &engine.output_dir,
                    )
                {
                    self.resolve_recorded_output_path(&engine.output_dir)
                } else {
                    Err(output_folder_not_recorded(&engine.output_dir))
                }
            }
        }
    }

    fn resolve_recorded_output_path(&self, candidate: &Path) -> Result<PathBuf, HubError> {
        validate_output_path_shape(candidate).map_err(|_| output_folder_not_recorded(candidate))?;

        if !candidate.is_dir() {
            return Err(output_folder_does_not_exist(candidate));
        }
        if path_contains_reparse_component(candidate)
            .map_err(|_| output_folder_not_recorded(candidate))?
        {
            return Err(output_folder_not_recorded(candidate));
        }
        ensure_no_reparse_directory(candidate)
            .map_err(|_| output_folder_not_recorded(candidate))?;

        let canonical = crate::projects::normalize_project_root(candidate);
        if !canonical.is_dir()
            || path_contains_reparse_component(&canonical)
                .map_err(|_| output_folder_not_recorded(candidate))?
        {
            return Err(output_folder_not_recorded(candidate));
        }
        ensure_no_reparse_directory(&canonical)
            .map_err(|_| output_folder_not_recorded(candidate))?;

        let trusted_roots = self.recorded_output_roots();
        if trusted_roots
            .iter()
            .any(|root| trusted_root_contains(root, &canonical))
        {
            Ok(canonical)
        } else {
            Err(output_folder_not_recorded(candidate))
        }
    }

    fn resolve_generated_output_path(&self, candidate: &Path) -> Result<PathBuf, HubError> {
        validate_output_path_shape(candidate).map_err(|_| output_folder_not_recorded(candidate))?;
        if !candidate.is_dir() {
            return Err(output_folder_does_not_exist(candidate));
        }
        if path_contains_reparse_component(candidate)
            .map_err(|_| output_folder_not_recorded(candidate))?
        {
            return Err(output_folder_not_recorded(candidate));
        }
        ensure_no_reparse_directory(candidate)
            .map_err(|_| output_folder_not_recorded(candidate))?;
        let canonical = crate::projects::normalize_project_root(candidate);
        if !canonical.is_dir()
            || path_contains_reparse_component(&canonical)
                .map_err(|_| output_folder_not_recorded(candidate))?
        {
            return Err(output_folder_not_recorded(candidate));
        }
        ensure_no_reparse_directory(&canonical)
            .map_err(|_| output_folder_not_recorded(candidate))?;
        Ok(canonical)
    }

    fn recorded_output_roots(&self) -> Vec<PathBuf> {
        let mut roots = Vec::new();
        if self
            .config
            .settings
            .default_build_output_grants_open_capability()
        {
            roots.push(self.config.settings.default_build_output_dir.clone());
        }
        if self
            .config
            .settings
            .default_device_install_grants_open_capability()
        {
            roots.push(self.config.settings.default_device_install_dir.clone());
        }
        roots
    }

    fn record_output_folder_failure(
        &mut self,
        target: String,
        detail: HubMessage,
        recovery: HubMessage,
    ) -> Result<(), HubError> {
        crate::state::push_action_record(
            &mut self.config.action_history,
            HubActionRecord {
                finished_unix_ms: crate::projects::now_unix_ms(),
                action: HubActionKind::OpenOutput,
                status: HubActionStatus::Failed,
                target: target.clone(),
                detail: detail.clone(),
                log_excerpt: detail.clone(),
                recovery: Some(recovery.clone()),
                process_id: None,
                command_line: Vec::new(),
                output_dir: None,
            },
        );
        self.task_status = TaskStatus::error("Open Output failed", detail, recovery)
            .with_operation(TaskOperationKind::Process, target);
        self.persist()
    }
}

fn action_history_id(record: &HubActionRecord) -> String {
    format!(
        "{}:{}:{}",
        record.finished_unix_ms,
        record.action.id(),
        record.target
    )
}

fn validate_output_path_shape(path: &Path) -> Result<(), ()> {
    if path.as_os_str().is_empty() || !path.is_absolute() {
        return Err(());
    }
    if path.components().any(|component| {
        matches!(component, Component::ParentDir)
            || matches!(component, Component::Normal(name) if name.is_empty())
    }) {
        return Err(());
    }

    #[cfg(windows)]
    {
        let is_local_disk = matches!(
            path.components().next(),
            Some(Component::Prefix(prefix))
                if matches!(prefix.kind(), std::path::Prefix::Disk(_))
        );
        if !is_local_disk
            || path.components().any(|component| {
                matches!(component, Component::Normal(name) if name.encode_wide().any(|unit| unit == b':' as u16))
            })
        {
            return Err(());
        }
    }

    Ok(())
}

fn path_contains_reparse_component(path: &Path) -> std::io::Result<bool> {
    let mut current = path.to_path_buf();
    loop {
        match fs::symlink_metadata(&current) {
            Ok(metadata) if metadata_is_reparse(&metadata) => return Ok(true),
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }

        let Some(parent) = current.parent().map(Path::to_path_buf) else {
            break;
        };
        if parent == current {
            break;
        }
        current = parent;
    }
    Ok(false)
}

#[cfg(windows)]
fn metadata_is_reparse(metadata: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;

    metadata.file_type().is_symlink() || metadata.file_attributes() & 0x0000_0400 != 0
}

#[cfg(not(windows))]
fn metadata_is_reparse(metadata: &fs::Metadata) -> bool {
    metadata.file_type().is_symlink()
}

fn ensure_no_reparse_directory(path: &Path) -> std::io::Result<()> {
    #[cfg(windows)]
    {
        let handle = unsafe {
            crate::file_io::windows::open_no_reparse(
                path,
                0x80, // FILE_READ_ATTRIBUTES
                0x7,  // FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE
                crate::file_io::windows::FILE_OPEN,
                true,
                std::ptr::null_mut(),
            )
        }?;
        use std::os::windows::fs::MetadataExt;
        let metadata = handle.metadata()?;
        if !metadata.is_dir() || metadata.file_attributes() & 0x0000_0400 != 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "output directory is a reparse point",
            ));
        }
    }

    #[cfg(not(windows))]
    if path_contains_reparse_component(path)? {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "output directory contains a symbolic link",
        ));
    }

    Ok(())
}

fn trusted_root_contains(root: &Path, candidate: &Path) -> bool {
    if validate_output_path_shape(root).is_err() || !root.is_dir() {
        return false;
    }
    if path_contains_reparse_component(root).unwrap_or(true)
        || ensure_no_reparse_directory(root).is_err()
    {
        return false;
    }

    let canonical_root = crate::projects::normalize_project_root(root);
    if !canonical_root.is_dir() || path_contains_reparse_component(&canonical_root).unwrap_or(true)
    {
        return false;
    }
    // Package/install receipts are children of the configured output root.
    // Containment is component-aware (never a textual prefix), and both
    // sides were checked for reparse points above. A successful legacy
    // history record is deliberately not itself a trust root: otherwise an
    // old arbitrary-path record could self-authorize a shell handoff.
    candidate.starts_with(&canonical_root)
        || cfg!(windows) && windows_path_contains(&canonical_root, candidate)
}

#[cfg(windows)]
fn windows_path_contains(root: &Path, candidate: &Path) -> bool {
    let normalize = |path: &Path| {
        path.to_string_lossy()
            .replace('/', "\\")
            .trim_end_matches('\\')
            .to_ascii_lowercase()
    };
    let root = normalize(root);
    let candidate = normalize(candidate);
    candidate == root
        || candidate
            .strip_prefix(&root)
            .is_some_and(|suffix| suffix.starts_with('\\'))
}

#[cfg(not(windows))]
fn windows_path_contains(_root: &Path, _candidate: &Path) -> bool {
    false
}

fn output_folder_does_not_exist(path: &Path) -> HubError {
    HubError::status(
        HubMessage::with_params(
            HubMessageId::Delivery(DeliveryMessageId::OutputFolderDoesNotExist),
            [path.to_string_lossy().into_owned()],
        ),
        Some(HubMessage::new(HubMessageId::Delivery(
            DeliveryMessageId::RunWorkflowAgainRecovery,
        ))),
    )
}

fn output_folder_not_recorded(path: &Path) -> HubError {
    HubError::status(
        HubMessage::with_params(
            HubMessageId::Delivery(DeliveryMessageId::OutputFolderNotRecorded),
            [path.to_string_lossy().into_owned()],
        ),
        Some(HubMessage::new(HubMessageId::Delivery(
            DeliveryMessageId::ChooseRecordedOutputRecovery,
        ))),
    )
}

#[cfg(test)]
#[path = "tests/output_actions.rs"]
mod tests;
