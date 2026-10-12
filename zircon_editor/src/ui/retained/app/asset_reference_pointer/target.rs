#[path = "target/dispatch.rs"]
mod dispatch;
#[path = "target/prepare.rs"]
mod prepare;
#[path = "target/state.rs"]
mod state;

use std::sync::Arc;

use super::super::UiSize;
use crate::ui::workbench::snapshot::AssetWorkspaceSnapshot;

pub(in crate::ui::retained_host::app) struct PreparedAssetReferencePointerTarget {
    pub snapshot: Arc<AssetWorkspaceSnapshot>,
    list_size: UiSize,
}
