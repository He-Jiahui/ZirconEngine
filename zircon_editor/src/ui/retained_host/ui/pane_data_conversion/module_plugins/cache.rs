use std::collections::BTreeMap;

use crate::ui::layouts::windows::workbench_host_window::ModulePluginStatusViewData;
use crate::ui::retained_host::host_contract::ModulePluginsPaneData;
use crate::ui::retained_host::primitives::{ModelRc, SharedString};

const MAX_CACHED_MODULE_PLUGIN_PANES: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ModulePluginsPaneProjectionCacheKey {
    pub document_identity: usize,
    pub uses_template: bool,
    pub width_bits: u32,
    pub height_bits: u32,
}

#[derive(Clone)]
struct ModulePluginsPaneProjectionCacheEntry {
    key: ModulePluginsPaneProjectionCacheKey,
    source_plugins: ModelRc<ModulePluginStatusViewData>,
    pane: ModulePluginsPaneData,
    last_used: u64,
}

#[derive(Default)]
pub(crate) struct ModulePluginsPaneProjectionCache {
    entries: BTreeMap<String, ModulePluginsPaneProjectionCacheEntry>,
    use_sequence: u64,
}

impl ModulePluginsPaneProjectionCache {
    pub(super) fn cached(
        &mut self,
        pane_id: &str,
        key: ModulePluginsPaneProjectionCacheKey,
        plugins: &ModelRc<ModulePluginStatusViewData>,
        diagnostics: &SharedString,
    ) -> Option<ModulePluginsPaneData> {
        self.use_sequence = self.use_sequence.wrapping_add(1);
        let last_used = self.use_sequence;
        let entry = self.entries.get_mut(pane_id)?;
        if entry.key != key
            || !entry.source_plugins.shares_values_with(plugins)
            || entry.pane.diagnostics.as_str() != diagnostics.as_str()
        {
            return None;
        }
        entry.last_used = last_used;
        Some(entry.pane.clone())
    }

    pub(super) fn store(
        &mut self,
        pane_id: String,
        key: ModulePluginsPaneProjectionCacheKey,
        source_plugins: ModelRc<ModulePluginStatusViewData>,
        pane: ModulePluginsPaneData,
    ) {
        self.use_sequence = self.use_sequence.wrapping_add(1);
        if !self.entries.contains_key(&pane_id)
            && self.entries.len() >= MAX_CACHED_MODULE_PLUGIN_PANES
        {
            let evicted = self
                .entries
                .iter()
                .min_by_key(|(_, entry)| entry.last_used)
                .map(|(pane_id, _)| pane_id.clone());
            if let Some(evicted) = evicted {
                self.entries.remove(&evicted);
            }
        }
        self.entries.insert(
            pane_id,
            ModulePluginsPaneProjectionCacheEntry {
                key,
                source_plugins,
                pane,
                last_used: self.use_sequence,
            },
        );
    }
}
