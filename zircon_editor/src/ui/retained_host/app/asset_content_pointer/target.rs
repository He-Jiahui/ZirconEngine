mod dispatch;
mod prepare;
mod state;

use std::sync::Arc;

use super::super::UiSize;
use crate::ui::workbench::snapshot::AssetWorkspaceSnapshot;

pub(in crate::ui::retained_host::app) struct PreparedAssetContentPointerTarget {
    pub snapshot: Arc<AssetWorkspaceSnapshot>,
    content_size: UiSize,
}
