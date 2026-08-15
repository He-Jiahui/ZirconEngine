use std::sync::{Arc, Mutex, MutexGuard};

use crate::core::asset::AssetTypeRegistry;
use crate::core::editor_event::{ConsoleMessageFilter, ConsoleSourceFilter};
use crate::core::extension::{ContributionStore, ContributionTicket};
use crate::ui::control::EditorUiControlService;
use crate::ui::host::EditorManager;
use crate::ui::workbench::reflection::EditorTransientUiState;
use crate::ui::workbench::state::EditorState;

/// UI-only workbench authority removed from the headless editor core.
pub(crate) struct WorkbenchShellState {
    state: Mutex<WorkbenchShellStateData>,
}

pub(crate) struct WorkbenchShellStateData {
    pub(crate) state: EditorState,
    pub(crate) manager: Arc<EditorManager>,
    pub(crate) transient: EditorTransientUiState,
    pub(crate) control_service: EditorUiControlService,
    pub(crate) contributions: ContributionStore,
    pub(crate) contribution_owners: Vec<OwnedContribution>,
    pub(crate) asset_type_registry_cache: AssetTypeRegistryGenerationCache,
    pub(crate) console_message_filter: ConsoleMessageFilter,
    pub(crate) console_source_filter: ConsoleSourceFilter,
}

impl WorkbenchShellStateData {
    pub(crate) fn contributions_changed(&mut self) {
        self.asset_type_registry_cache.contributions_changed();
    }

    pub(crate) fn set_console_source_filter(&mut self, filter: ConsoleSourceFilter) -> bool {
        let changed = self.console_source_filter != filter;
        self.console_source_filter = filter;
        changed
    }

    pub(crate) fn set_console_message_filter(&mut self, filter: ConsoleMessageFilter) -> bool {
        let changed = self.console_message_filter != filter;
        self.console_message_filter = filter;
        changed
    }
}

pub(crate) struct OwnedContribution {
    owner_id: String,
    ticket: ContributionTicket,
}

impl OwnedContribution {
    pub(crate) fn new(owner_id: impl Into<String>, ticket: ContributionTicket) -> Self {
        Self {
            owner_id: owner_id.into(),
            ticket,
        }
    }

    pub(crate) fn owner_id(&self) -> &str {
        &self.owner_id
    }

    pub(crate) fn ticket(&self) -> ContributionTicket {
        self.ticket
    }
}

#[derive(Default)]
pub(crate) struct AssetTypeRegistryGenerationCache {
    extension_generation: u64,
    cached: Option<CachedAssetTypeRegistry>,
    cache_hits: u64,
    materializations: u64,
}

struct CachedAssetTypeRegistry {
    extension_generation: u64,
    enabled_capabilities: Vec<String>,
    registry: Arc<AssetTypeRegistry>,
}

impl AssetTypeRegistryGenerationCache {
    pub(crate) fn contributions_changed(&mut self) {
        self.extension_generation = self.extension_generation.saturating_add(1);
        self.cached = None;
    }

    pub(crate) fn get(
        &mut self,
        enabled_capabilities: &[String],
    ) -> Option<Arc<AssetTypeRegistry>> {
        let cached = self.cached.as_ref().filter(|cached| {
            cached.extension_generation == self.extension_generation
                && cached.enabled_capabilities.as_slice() == enabled_capabilities
        })?;
        let registry = Arc::clone(&cached.registry);
        self.cache_hits = self.cache_hits.saturating_add(1);
        Some(registry)
    }

    pub(crate) fn store(
        &mut self,
        enabled_capabilities: Vec<String>,
        registry: Arc<AssetTypeRegistry>,
    ) {
        self.materializations = self.materializations.saturating_add(1);
        self.cached = Some(CachedAssetTypeRegistry {
            extension_generation: self.extension_generation,
            enabled_capabilities,
            registry,
        });
    }

    #[cfg(test)]
    pub(crate) fn counts(&self) -> (u64, u64) {
        (self.cache_hits, self.materializations)
    }
}

impl WorkbenchShellState {
    pub(crate) fn new(state: EditorState, manager: Arc<EditorManager>) -> Self {
        Self {
            state: Mutex::new(WorkbenchShellStateData {
                state,
                manager,
                transient: EditorTransientUiState::default(),
                control_service: EditorUiControlService::default(),
                contributions: ContributionStore::default(),
                contribution_owners: Vec::new(),
                asset_type_registry_cache: AssetTypeRegistryGenerationCache::default(),
                console_message_filter: ConsoleMessageFilter::default(),
                console_source_filter: ConsoleSourceFilter::default(),
            }),
        }
    }

    pub(crate) fn lock(&self) -> MutexGuard<'_, WorkbenchShellStateData> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
