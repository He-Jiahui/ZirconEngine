use std::collections::BTreeSet;
use std::fmt;

use serde::{Deserialize, Deserializer, Serialize};

/// Ordered facts that must all be true before Hub can treat an Editor as interactive.
///
/// The receipt deliberately does not expose a project path. Its mailbox location already scopes it
/// to one project, while the launch session binds it to the Hub child process that requested it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HubEditorStartupMilestoneV1 {
    SessionCommitted,
    NativeWindowCreated,
    FirstPresent,
    FocusInboxBound,
    Interactive,
}

impl HubEditorStartupMilestoneV1 {
    const REQUIRED: [Self; 5] = [
        Self::SessionCommitted,
        Self::NativeWindowCreated,
        Self::FirstPresent,
        Self::FocusInboxBound,
        Self::Interactive,
    ];

    fn required_set() -> BTreeSet<Self> {
        Self::REQUIRED.into_iter().collect()
    }
}

/// A path-redacted proof that a Hub-launched Editor has completed the startup milestones.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct HubEditorReadyReceiptV1 {
    editor_process_id: u32,
    editor_instance_id: String,
    session_generation: u64,
    milestones: BTreeSet<HubEditorStartupMilestoneV1>,
}

impl HubEditorReadyReceiptV1 {
    /// Creates the only Ready receipt shape: a committed session, created native window, submitted
    /// first frame, generation-qualified focus inbox, and interactive Editor.
    pub fn after_first_present(
        editor_process_id: u32,
        editor_instance_id: impl Into<String>,
        session_generation: u64,
    ) -> Result<Self, HubEditorReadyReceiptError> {
        Self::from_parts(
            editor_process_id,
            editor_instance_id,
            session_generation,
            HubEditorStartupMilestoneV1::required_set(),
        )
    }

    pub const fn editor_process_id(&self) -> u32 {
        self.editor_process_id
    }

    pub fn editor_instance_id(&self) -> &str {
        &self.editor_instance_id
    }

    pub const fn session_generation(&self) -> u64 {
        self.session_generation
    }

    pub fn milestones(&self) -> &BTreeSet<HubEditorStartupMilestoneV1> {
        &self.milestones
    }

    fn from_parts(
        editor_process_id: u32,
        editor_instance_id: impl Into<String>,
        session_generation: u64,
        milestones: BTreeSet<HubEditorStartupMilestoneV1>,
    ) -> Result<Self, HubEditorReadyReceiptError> {
        if editor_process_id == 0 {
            return Err(HubEditorReadyReceiptError::ZeroProcessId);
        }
        let editor_instance_id = editor_instance_id.into();
        if editor_instance_id.is_empty() || editor_instance_id.len() > MAX_EDITOR_INSTANCE_ID_BYTES
        {
            return Err(HubEditorReadyReceiptError::InvalidEditorInstanceId);
        }
        if session_generation == 0 {
            return Err(HubEditorReadyReceiptError::ZeroSessionGeneration);
        }
        if milestones != HubEditorStartupMilestoneV1::required_set() {
            return Err(HubEditorReadyReceiptError::IncompleteMilestones);
        }
        Ok(Self {
            editor_process_id,
            editor_instance_id,
            session_generation,
            milestones,
        })
    }
}

const MAX_EDITOR_INSTANCE_ID_BYTES: usize = 128;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct HubEditorReadyReceiptWireV1 {
    editor_process_id: u32,
    editor_instance_id: String,
    session_generation: u64,
    milestones: BTreeSet<HubEditorStartupMilestoneV1>,
}

impl<'de> Deserialize<'de> for HubEditorReadyReceiptV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = HubEditorReadyReceiptWireV1::deserialize(deserializer)?;
        Self::from_parts(
            wire.editor_process_id,
            wire.editor_instance_id,
            wire.session_generation,
            wire.milestones,
        )
        .map_err(serde::de::Error::custom)
    }
}

/// Receipt validation failures are deliberately categorical so they can be reported without
/// leaking project-specific diagnostics through the shared mailbox.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HubEditorReadyReceiptError {
    ZeroProcessId,
    InvalidEditorInstanceId,
    ZeroSessionGeneration,
    IncompleteMilestones,
}

impl fmt::Display for HubEditorReadyReceiptError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::ZeroProcessId => "editor process id must be non-zero",
            Self::InvalidEditorInstanceId => "editor instance id must be non-empty and bounded",
            Self::ZeroSessionGeneration => "editor session generation must be non-zero",
            Self::IncompleteMilestones => "editor Ready receipt requires every startup milestone",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for HubEditorReadyReceiptError {}

#[cfg(test)]
mod tests {
    use super::{HubEditorReadyReceiptV1, HubEditorStartupMilestoneV1};

    #[test]
    fn receipt_after_first_present_contains_every_required_milestone() {
        let receipt =
            HubEditorReadyReceiptV1::after_first_present(913, "913-42", 7).expect("valid receipt");

        assert_eq!(receipt.milestones().len(), 5);
        assert!(receipt
            .milestones()
            .contains(&HubEditorStartupMilestoneV1::FirstPresent));
    }

    #[test]
    fn deserialize_rejects_partial_or_path_bearing_ready_receipts() {
        assert!(serde_json::from_str::<HubEditorReadyReceiptV1>(
            r#"{"editor_process_id":913,"editor_instance_id":"913-42","session_generation":7,"milestones":["session_committed","native_window_created"]}"#,
        )
        .is_err());
        assert!(serde_json::from_str::<HubEditorReadyReceiptV1>(
            r#"{"editor_process_id":913,"editor_instance_id":"913-42","session_generation":7,"milestones":["session_committed","native_window_created","first_present","focus_inbox_bound","interactive"],"project":"E:/Projects/Secret"}"#,
        )
        .is_err());
    }
}
