use serde::{Deserialize, Serialize};

use super::UiComponentProjectionPatch;
use crate::ui::component::UiValidationState;

/// Reports adapter mutation status and optional projection updates to the host UI.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiComponentAdapterResult {
    pub changed: bool,
    pub refresh_projection: bool,
    pub dirty: bool,
    pub transaction_id: Option<String>,
    pub mutation_source: Option<String>,
    pub status_text: Option<String>,
    pub validation: Option<UiValidationState>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub patches: Vec<UiComponentProjectionPatch>,
}

impl UiComponentAdapterResult {
    pub fn unchanged() -> Self {
        Self::default()
    }

    pub fn changed() -> Self {
        Self {
            changed: true,
            refresh_projection: true,
            dirty: true,
            ..Self::default()
        }
    }

    pub fn dirty(mut self, dirty: bool) -> Self {
        self.dirty = dirty;
        self
    }

    pub fn with_transaction(mut self, transaction_id: impl Into<String>) -> Self {
        self.transaction_id = Some(transaction_id.into());
        self
    }

    pub fn with_mutation_source(mut self, mutation_source: impl Into<String>) -> Self {
        self.mutation_source = Some(mutation_source.into());
        self
    }

    pub fn with_status(mut self, status_text: impl Into<String>) -> Self {
        self.status_text = Some(status_text.into());
        self
    }

    pub fn with_patch(mut self, patch: UiComponentProjectionPatch) -> Self {
        self.patches.push(patch);
        self
    }
}
