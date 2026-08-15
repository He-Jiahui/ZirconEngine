use serde::{Deserialize, Serialize};

use super::{EditorUiDeltaBatch, ViewDirtySet};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EditorViewRefreshReport {
    dirty: ViewDirtySet,
    #[serde(default)]
    deltas: EditorUiDeltaBatch,
    used_full_snapshot_fallback: bool,
}

impl EditorViewRefreshReport {
    pub fn new(
        dirty: ViewDirtySet,
        deltas: EditorUiDeltaBatch,
        used_full_snapshot_fallback: bool,
    ) -> Self {
        Self {
            dirty,
            deltas,
            used_full_snapshot_fallback,
        }
    }

    pub fn dirty(&self) -> &ViewDirtySet {
        &self.dirty
    }

    pub fn deltas(&self) -> &EditorUiDeltaBatch {
        &self.deltas
    }

    pub fn used_full_snapshot_fallback(&self) -> bool {
        self.used_full_snapshot_fallback
    }
}
