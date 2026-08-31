use std::fmt;
use std::path::{Path, PathBuf};

use super::HubSessionToken;

const ZIRCON_DIRECTORY: &str = ".zircon";
const HUB_DIRECTORY: &str = "hub";
const FOCUS_DIRECTORY: &str = "focus";
const FOCUS_REQUESTS_DIRECTORY: &str = "requests";
const FOCUS_ACKS_DIRECTORY: &str = "acks";

/// Returns the project-owned, instance-addressed Hub focus mailbox path.
pub fn hub_editor_focus_signal_path(
    project_root: impl AsRef<Path>,
    target_instance_id: &str,
    sequence: u64,
    request_id: HubSessionToken,
) -> Result<PathBuf, HubEditorFocusSignalPathError> {
    Ok(
        hub_editor_focus_request_directory(project_root, target_instance_id)?
            .join(request_file_name(sequence, request_id)),
    )
}

/// Returns the instance-owned request inbox directory for sequenced Hub focus work.
pub fn hub_editor_focus_request_directory(
    project_root: impl AsRef<Path>,
    target_instance_id: &str,
) -> Result<PathBuf, HubEditorFocusSignalPathError> {
    Ok(
        hub_editor_focus_instance_directory(project_root, target_instance_id)?
            .join(FOCUS_REQUESTS_DIRECTORY),
    )
}

/// Returns the instance-owned acknowledgement directory for sequenced Hub focus work.
pub fn hub_editor_focus_ack_path(
    project_root: impl AsRef<Path>,
    target_instance_id: &str,
    sequence: u64,
    request_id: HubSessionToken,
) -> Result<PathBuf, HubEditorFocusSignalPathError> {
    Ok(
        hub_editor_focus_instance_directory(project_root, target_instance_id)?
            .join(FOCUS_ACKS_DIRECTORY)
            .join(request_file_name(sequence, request_id)),
    )
}

fn hub_editor_focus_instance_directory(
    project_root: impl AsRef<Path>,
    target_instance_id: &str,
) -> Result<PathBuf, HubEditorFocusSignalPathError> {
    validate_instance_id(target_instance_id)?;
    Ok(project_root
        .as_ref()
        .join(ZIRCON_DIRECTORY)
        .join(HUB_DIRECTORY)
        .join(FOCUS_DIRECTORY)
        .join(target_instance_id))
}

fn request_file_name(sequence: u64, request_id: HubSessionToken) -> String {
    format!("{sequence:020}-{request_id}.json")
}

/// The target instance identity is part of the mailbox filename and must remain path-safe.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HubEditorFocusSignalPathError {
    instance_id: String,
}

impl HubEditorFocusSignalPathError {
    fn new(instance_id: impl Into<String>) -> Self {
        Self {
            instance_id: instance_id.into(),
        }
    }

    pub fn instance_id(&self) -> &str {
        &self.instance_id
    }
}

impl fmt::Display for HubEditorFocusSignalPathError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "invalid editor session instance id `{}`",
            self.instance_id
        )
    }
}

impl std::error::Error for HubEditorFocusSignalPathError {}

fn validate_instance_id(instance_id: &str) -> Result<(), HubEditorFocusSignalPathError> {
    if instance_id.is_empty()
        || !instance_id
            .bytes()
            .all(|byte| byte.is_ascii_digit() || byte == b'-')
    {
        return Err(HubEditorFocusSignalPathError::new(instance_id));
    }
    Ok(())
}
