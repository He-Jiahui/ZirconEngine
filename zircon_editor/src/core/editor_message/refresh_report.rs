use serde::{Deserialize, Serialize};

use super::ViewDirtySet;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorViewRefreshReport {
    dirty: ViewDirtySet,
    used_full_snapshot_fallback: bool,
}

impl EditorViewRefreshReport {
    pub fn new(dirty: ViewDirtySet, used_full_snapshot_fallback: bool) -> Self {
        Self {
            dirty,
            used_full_snapshot_fallback,
        }
    }

    pub fn dirty(&self) -> &ViewDirtySet {
        &self.dirty
    }

    pub fn used_full_snapshot_fallback(&self) -> bool {
        self.used_full_snapshot_fallback
    }
}
