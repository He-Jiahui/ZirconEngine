use std::sync::{Mutex, MutexGuard};

use crate::core::asset::EditorAssetIndex;

pub(super) fn lock_editor_asset_index_recovering_poison(
    index: &Mutex<EditorAssetIndex>,
) -> MutexGuard<'_, EditorAssetIndex> {
    index
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}
