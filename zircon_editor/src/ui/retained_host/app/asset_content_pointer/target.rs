mod dispatch;
mod prepare;
mod state;

use super::super::UiSize;
use crate::ui::workbench::snapshot::AssetWorkspaceSnapshot;

pub(in crate::ui::retained_host::app) struct PreparedAssetContentPointerTarget {
    pub snapshot: AssetWorkspaceSnapshot,
    content_size: UiSize,
}
