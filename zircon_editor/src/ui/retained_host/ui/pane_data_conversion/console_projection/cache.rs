use std::collections::BTreeMap;

use crate::ui::retained_host::ConsolePaneData;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ConsolePaneProjectionCacheKey {
    pub document_identity: usize,
    pub width_bits: u32,
    pub height_bits: u32,
}

#[derive(Clone)]
pub(super) struct ConsolePaneProjectionCacheEntry {
    pub key: ConsolePaneProjectionCacheKey,
    pub pane: ConsolePaneData,
}

#[derive(Default)]
pub(crate) struct ConsolePaneProjectionCache {
    entries: BTreeMap<String, ConsolePaneProjectionCacheEntry>,
}

impl ConsolePaneProjectionCache {
    pub(super) fn get(
        &self,
        pane_id: &str,
        key: ConsolePaneProjectionCacheKey,
    ) -> Option<&ConsolePaneProjectionCacheEntry> {
        self.entries.get(pane_id).filter(|entry| entry.key == key)
    }

    pub(super) fn publish(
        &mut self,
        pane_id: String,
        key: ConsolePaneProjectionCacheKey,
        pane: ConsolePaneData,
    ) {
        self.entries
            .insert(pane_id, ConsolePaneProjectionCacheEntry { key, pane });
    }
}
