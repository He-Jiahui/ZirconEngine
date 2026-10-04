//! 统一导入工作与提交端访问资源索引的中毒恢复策略；调用端仍必须遵守共享航班与索引状态迁移的顺序。
use std::sync::{Mutex, MutexGuard};

use crate::core::asset::EditorAssetIndex;

pub(super) fn lock_editor_asset_index_recovering_poison(
    index: &Mutex<EditorAssetIndex>,
) -> MutexGuard<'_, EditorAssetIndex> {
    index
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}
