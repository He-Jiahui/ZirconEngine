use serde::{Deserialize, Serialize};

use super::{EditorUiDeltaBatch, ViewDirtySet};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
/// 宿主一次消费待刷新工作的结果，包含失效集合、增量补丁和是否使用完整反射回退。
/// 此报告描述此次刷新决策；后续新消息留待下一次消费。
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
