mod dispatch;
mod prepare;
mod state;

use super::super::UiSize;
use crate::ui::workbench::snapshot::AssetWorkspaceSnapshot;

pub(in crate::ui::retained_host::app) struct PreparedAssetReferencePointerTarget {
    pub snapshot: AssetWorkspaceSnapshot,
    list_size: UiSize,
}
