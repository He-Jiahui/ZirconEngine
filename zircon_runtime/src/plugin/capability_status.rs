use serde::{Deserialize, Serialize};

use crate::RuntimeTargetMode;

/// Implementation state for a plugin-owned capability or optional feature.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityStatus {
    Complete,
    Partial,
    Stub,
    Externalized,
    Unsupported,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityStatusManifest {
    pub capability: String,
    pub status: CapabilityStatus,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub target_modes: Vec<RuntimeTargetMode>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bevy_references: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

impl CapabilityStatusManifest {
    pub fn new(capability: impl Into<String>, status: CapabilityStatus) -> Self {
        Self {
            capability: capability.into(),
            status,
            target_modes: Vec::new(),
            bevy_references: Vec::new(),
            note: None,
        }
    }

    pub fn with_target_modes(
        mut self,
        target_modes: impl IntoIterator<Item = RuntimeTargetMode>,
    ) -> Self {
        self.target_modes = target_modes.into_iter().collect();
        self
    }

    pub fn with_bevy_reference(mut self, reference: impl Into<String>) -> Self {
        self.bevy_references.push(reference.into());
        self
    }

    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.note = Some(note.into());
        self
    }
}
